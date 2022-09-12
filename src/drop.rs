use alloc::alloc::{Allocator, Global, Layout};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ptr;

#[cfg(doc)]
use crate::adopt::Adopt;
use crate::graph::Graph;
use crate::hash::HashSet;
use crate::link::{Kind, Link};
use crate::rc::RcInnerPtr;
use crate::Rc;

unsafe impl<#[may_dangle] T> Drop for Rc<T> {
    /// Drops the [`Rc`].
    ///
    /// This will decrement the strong reference count. If the strong reference
    /// count reaches zero then the only other references (if any) are [`Weak`],
    /// so we `drop` the inner value.
    ///
    /// [`Weak`]: crate::Weak
    ///
    /// If this `Rc` has adopted any other `Rc`s, drop will trace the reachable
    /// object graph and detect if this `Rc` is part of an orphaned cycle. An
    /// orphaned cycle is a cycle in which all members have no owned references
    /// held by `Rc`s outside of the cycle.
    ///
    /// `Rc`s do not pay the cost of the reachability check unless they use
    /// [`Adopt::adopt_unchecked`].
    ///
    /// [`Adopt::adopt_unchecked`]: crate::Adopt::adopt_unchecked
    ///
    /// # Examples
    ///
    /// ```
    /// use cactusref::Rc;
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo  = Rc::new(Foo);
    /// let foo2 = Rc::clone(&foo);
    ///
    /// drop(foo);    // Doesn't print anything
    /// drop(foo2);   // Prints "dropped!"
    /// ```
    ///
    /// ```
    /// use cactusref::{Adopt, Rc};
    ///
    /// struct Foo(u8);
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped {}!", self.0);
    ///     }
    /// }
    ///
    /// let foo  = Rc::new(Foo(10));
    /// let foo2 = Rc::new(Foo(20));
    ///
    /// unsafe {
    ///     Rc::adopt_unchecked(&foo, &foo2);
    ///     Rc::adopt_unchecked(&foo2, &foo);
    /// }
    ///
    /// drop(foo);    // Doesn't print anything
    /// drop(foo2);   // Prints "dropped 10!" and "dropped 20!"
    /// ```
    ///
    /// # Cycle Detection and Deallocation Algorithm
    ///
    /// [`Rc::adopt_unchecked`] does explicit bookkeeping to store links to
    /// adoptee `Rc`s.  These links form a graph of reachable objects which are
    /// used to detect cycles.
    ///
    /// [`Rc::adopt_unchecked`]: crate::Rc::adopt_unchecked
    ///
    /// On drop, if an `Rc` has no links, it is dropped like a normal `Rc`. If
    /// the `Rc` has links, `Drop` performs a breadth first search by traversing
    /// the forward and backward links stored in each `Rc`. Deallocating cycles
    /// requires correct use of [`Adopt::adopt_unchecked`] and [`Adopt::unadopt`]
    /// to perform the reachability bookkeeping.
    ///
    /// [`Adopt::adopt_unchecked`]: crate::Adopt::adopt_unchecked
    /// [`Adopt::unadopt`]: crate::Adopt::unadopt
    ///
    /// After determining all reachable objects, `Rc` reduces the graph to
    /// objects that form a cycle by performing pairwise reachability checks.
    /// During this step, for each object in the cycle, `Rc` counts the number
    /// of refs held by other objects in the cycle.
    ///
    /// Using the cycle-held references, `Rc` computes whether the object graph
    /// is reachable by any non-cycle nodes by comparing strong counts.
    ///
    /// If the cycle is orphaned, `Rc` busts all the link structures and
    /// deallocates each object.
    ///
    /// ## Performance
    ///
    /// Cycle detection uses breadth first search to trace the object graph.
    /// The runtime complexity of detecting a cycle is `O(links + nodes)` where
    /// links is the number of adoptions that are alive and nodes is the number
    /// of objects in the cycle.
    ///
    /// Determining whether the cycle is orphaned builds on cycle detection and
    /// iterates over all nodes in the graph to see if their strong count is
    /// greater than the number of references in the cycle. The runtime
    /// complexity of finding an orphaned cycle is `O(links + nodes)` where
    /// links is the number of adoptions that are alive and nodes is the number
    /// objects in the cycle.
    fn drop(&mut self) {
        // If `self` is held in a cycle, as we deallocate members of the cycle,
        // they will drop their refs to `self`. To prevent a double free, mark
        // nodes as dead if they have already been deallocated and short
        // circuit.
        if self.inner().is_dead() {
            return;
        }

        // If a drop is occuring it is because there was an existing `Rc` which
        // is maintaining a strong count. Decrement the strong count on drop,
        // even if this `Rc` is dead. This ensures `Weak::upgrade` behaves
        // correctly for deallocated cycles and does not cause a use-after-free.
        self.inner().dec_strong();

        // If inner has a graph pointer, it is part of an adoption chain or
        // cycle.
        if let Some(graph) = self.inner().graph.take() {
            std::dbg!(self.inner().strong());
            if std::dbg!(self.inner().is_dead()) {
                unsafe {
                    let graph = std::dbg!(Box::from_raw(graph.as_ptr()));
                    let mut graph = ManuallyDrop::new(graph);
                    drop_unreachable_with_adoptions(self, &mut graph);
                    if graph.is_empty() {
                        ManuallyDrop::drop(&mut graph);
                    }
                }
                return;
            }
            if unsafe { std::dbg!(std::dbg!(graph.as_ref()).is_externally_reachable()) } {
                self.inner().graph.set(Some(graph));
                return;
            }
            unsafe {
                self.inner().inc_strong();
                let graph = Box::from_raw(graph.as_ptr());
                std::dbg!();
                drop_cycle(graph);
            }
            return;
        }
        // If inner *does not* have a graph pointer, the object is just a normal
        // `Rc` and we can drop and deallocate if it is dead.
        if self.inner().is_dead() {
            unsafe {
                drop_unreachable(self);
            }
            return;
        }
        debug!("cactusref drop skipped, Rc is reachable");
    }
}

unsafe fn drop_unreachable<T>(this: &mut Rc<T>) {
    debug!("cactusref detected unreachable Rc");

    let rcbox = this.ptr.as_ptr();
    // Mark `this` as pending deallocation. This is not strictly necessary since
    // `this` is unreachable, but `kill`ing `this ensures we don't double-free.
    if !(*rcbox).is_uninit() {
        trace!("cactusref deallocating unreachable RcBox {:p}", rcbox);
        // Mark the `RcBox` as uninitialized so we can make its `MaybeUninit`
        // fields uninhabited.
        (*rcbox).make_uninit();

        // Move `T` out of the `RcBox`. Dropping an uninitialized `MaybeUninit`
        // has no effect.
        let inner = mem::replace(&mut (*rcbox).value, MaybeUninit::uninit());
        // destroy the contained `T`.
        drop(inner.assume_init());
    }

    // remove the implicit "strong weak" pointer now that we've destroyed the
    // contents.
    (*rcbox).dec_weak();

    if (*rcbox).weak() == 0 {
        // SAFETY: `T` is `Sized`, which means `Layout::for_value_raw` is always
        // safe to call.
        let layout = Layout::for_value_raw(this.ptr.as_ptr());
        Global.deallocate(this.ptr.cast(), layout);
    }
}

unsafe fn drop_cycle<T>(graph: Box<Graph<T>>) {
    debug!(
        "cactusref detected orphaned cycle with {} objects",
        graph.len()
    );

    // Iterate over all the nodes in the cycle, bust all of the links. All nodes
    // in the cycle are reachable by other nodes in the cycle, so removing
    // all cycle-internal links won't result in a leak.
    for &(src, _) in &graph.edges {
        if (*src.as_ptr()).is_dead() {
            continue;
        }
        trace!("cactusref dropping {:?} member of orphaned cycle", src);

        // Remove reverse links so `this` is not included in cycle detection for
        // objects that had adopted `this`. This prevents a use-after-free in
        // `Rc::orphaned_cycle`.
        //
        // Because the entire cycle is unreachable, the only forward and
        // backward links are to objects in the cycle that we are about to
        // deallocate. This allows us to bust the cycle detection by clearing
        // all links.
        let cycle_strong_refs = std::dbg!(graph.count_directed_edges_toward(src.inner));
        let rcbox = src.as_ptr();

        // To be in a cycle, at least one `value` field in an `RcBox` in the
        // cycle holds a strong reference to `this`. Mark all nodes in the cycle
        // as dead so when we deallocate them via the `value` pointer we don't
        // get a double-free.
        for _ in 0..cycle_strong_refs.min((*rcbox).strong()) {
            (*rcbox).dec_strong();
        }
        std::dbg!((*rcbox).weak());
    }
    let mut inners = vec![];
    for &(node, _) in &graph.edges {
        let ptr = node.inner;
        let rcbox = ptr.as_ptr();
        if !(*rcbox).is_dead() {
            // This object continues to be referenced outside the cycle in
            // another part of the graph.
            continue;
        }

        if !(*rcbox).is_uninit() {
            // Mark the `RcBox` as uninitialized so we can make its
            // `MaybeUninit` fields uninhabited.
            (*rcbox).make_uninit();
            (*rcbox).graph = core::cell::Cell::new(None);

            // Move `T` out of the `RcBox`. Dropping an uninitialized
            // `MaybeUninit` has no effect.
            let inner = mem::replace(&mut (*rcbox).value, MaybeUninit::uninit());
            trace!("cactusref deconstructed member {:p} of orphan cycle", rcbox);
            // Move `T` out of the `RcBox` to be dropped after busting the cycle.
            inners.push(inner.assume_init());
        }
    }
    // Drop and deallocate all `T` and `HashMap` objects.
    drop(inners);

    let unreachable_cycle_participants = graph
        .edges
        .into_iter()
        .filter_map(|(left, right)| {
            if left.inner == right.inner {
                None
            } else {
                Some(left.inner)
            }
        })
        .filter(|ptr| {
            // Filter the set of cycle participants so we only drop `Rc`s that are
            // dead.
            //
            // If an `Rc` is not dead, it continues to be referenced outside of the
            // cycle, for example:
            //
            //  | Rc | -> | Rc | -> | Rc | <-> | Rc |
            //    ^                   |
            //    |-------------------|
            //
            // This object continues to be referenced outside the cycle in another
            // part of the graph.
            let rcbox = ptr.as_ptr();
            std::dbg!(rcbox);
            unsafe { (*rcbox).is_dead() }
        })
        .collect::<HashSet<_>>();

    for ptr in unreachable_cycle_participants {
        trace!(
            "cactusref deallocating RcBox after dropping item {:?} in orphaned cycle",
            ptr
        );

        let rcbox = std::dbg!(ptr).as_ptr();
        if (*rcbox).weak() == 0 {
            continue;
        }
        // remove the implicit "strong weak" pointer now that we've destroyed
        // the contents.
        (*rcbox).dec_weak();

        if (*rcbox).weak() == 0 {
            trace!(
                "no more weak references, deallocating layout for item {:?} in orphaned cycle",
                ptr
            );
            // SAFETY: `T` is `Sized`, which means `Layout::for_value_raw` is
            // always safe to call.
            let layout = Layout::for_value_raw(ptr.as_ptr());
            Global.deallocate(ptr.cast(), layout);
        }
    }
}

// Drop an `Rc` that is unreachable, but has adopted other `Rc`s.
//
// Unreachable `Rc`s have a strong count of zero, but because they have adopted
// other `Rc`s, other `Rc`s have back links to `this`.
//
// Before dropping `this`, we must traverse `this`'s forward links to collect
// all of `this`'s adoptions. Then, remove `this` from it's adoptions back
// links. By pruning back links in the rest of the graph, we can ensure that
// `this` and its `RcBox` are not referenced and can be safely deallocated.
//
// # Diagram
//
//          this
// |--------------------|
// | ptr:    RcBox      |
// |      |----------| <--------|
// |      | value: T |  |       |
// |      | links: ------> | other RcBox |
// |      |   |----------> | other RcBox |
// |      |          |  |       |
// |      |----------| <--------|
// |--------------------|
unsafe fn drop_unreachable_with_adoptions<T>(this: &mut Rc<T>, graph: &mut Box<Graph<T>>) {
    std::dbg!(this.ptr);
    let mut to_unadopt = Vec::with_capacity(graph.len());
    // `this` is unreachable but may have been adopted and dropped.
    //
    // Iterate over all of the other nodes in the graph that have links to
    // `this` and remove all of the adoptions. By doing so, when other graph
    // participants are dropped, they do not try to deallocate `this`.
    //
    // `this` is fully removed from the graph.
    for &(src, dst) in &graph.edges {
        if ptr::eq(dst.as_ptr(), this.inner()) {
            to_unadopt.push((src, dst));
        }
    }
    std::dbg!();
    for (src, dst) in to_unadopt {
        std::dbg!();
        graph.unlink(src.inner, dst.inner);
        std::dbg!();
    }
    // we're about to dealloc `this`, purge it from the graph.
    graph
        .edges
        .drain_filter(|&mut (_, dst)| dst.inner == this.ptr)
        .count();

    let rcbox = this.ptr.as_ptr();
    // Mark `this` as pending deallocation. This is not strictly necessary since
    // `this` is unreachable, but `kill`ing `this ensures we don't double-free.
    std::dbg!();
    if !(*rcbox).is_uninit() {
        trace!(
            "cactusref deallocating RcBox after dropping adopted and unreachable item {:p} in the object graph",
            rcbox
        );
        // Mark the `RcBox` as uninitialized so we can make its `MaybeUninit`
        // fields uninhabited.
        (*rcbox).make_uninit();
        std::dbg!();

        // Move `T` out of the `RcBox`. Dropping an uninitialized `MaybeUninit`
        // has no effect.
        let inner = mem::replace(&mut (*rcbox).value, MaybeUninit::uninit());
        std::dbg!();
        // destroy the contained `T`.
        drop(inner.assume_init());
        std::dbg!();
    }

    // remove the implicit "strong weak" pointer now that we've destroyed the
    // contents.
    (*rcbox).dec_weak();

    if (*rcbox).weak() == 0 {
        trace!(
            "no more weak references, deallocating layout for adopted and unreachable item {:?} in the object graph",
            this.ptr
        );
        // SAFETY: `T` is `Sized`, which means `Layout::for_value_raw` is always
        // safe to call.
        std::dbg!();
        let layout = Layout::for_value_raw(this.ptr.as_ptr());
        std::dbg!();
        Global.deallocate(this.ptr.cast(), layout);
        std::dbg!();
    }
}
