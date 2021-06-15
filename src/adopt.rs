use core::ptr;

use crate::link::Link;
use crate::trace::Trace;
use crate::Rc;

mod sealed {
    use crate::Rc;

    #[doc(hidden)]
    pub trait Sealed {}

    impl<T> Sealed for Rc<T> {}
}

/// Build a graph of linked [`Rc`] smart pointers to enable busting cycles on
/// drop.
///
/// Calling [`adopt_unchecked`] builds an object graph which can be used by to
/// detect cycles.
///
/// # Safety
///
/// Implementors of this trait must ensure that bookkeeping edges in the object
/// graph is correct because these links are used to determine whether an `Rc`
/// is reachable in `Rc`'s `Drop` implementation. Failure to properly bookkeep
/// the object graph will result in *[undefined behavior]*.
///
/// Undefined behavior may include:
///
/// - Memory leaks.
/// - Double-frees.
/// - Dangling `Rc`s which will cause a use after free.`
///
/// [`adopt_unchecked`]: Adopt::adopt_unchecked
/// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
pub unsafe trait Adopt: sealed::Sealed {
    /// The smart pointer's inner owned value.
    type Inner;

    /// TODO: document me!
    fn adopt(this: &mut Self, other: &Self)
    where
        Self::Inner: Trace;

    /// Perform bookkeeping to record that `this` has an owned reference to
    /// `other`.
    ///
    /// Adoption is a one-way link, or a directed edge in the object graph which
    /// means "`this` owns `other`".
    ///
    /// `adopt` can be called multiple times for a pair of `Rc`s. Each call to
    /// `adopt` indicates that `this` owns one distinct clone of `other`.
    ///
    /// This is an associated function that needs to be used as
    /// `Adopt::adopt_unchecked(...)`. A method would interfere with methods of the same
    /// name on the contents of a `Rc` used through `Deref`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `this` owns a strong reference to `other`.
    ///
    /// Callers should call [`unadopt`] when `this` no longer holds a strong
    /// reference to `other` to avoid memory leaks, but this is not required for
    /// soundness.
    ///
    /// [`unadopt`]: Adopt::unadopt
    unsafe fn adopt_unchecked(this: &Self, other: &Self);

    /// Perform bookkeeping to record that `this` has removed an owned reference
    /// to `other`.
    ///
    /// Adoption is a one-way link, or a directed edge in the object graph which
    /// means "`this` owns `other`".
    ///
    /// This is an associated function that needs to be used as
    /// `Adopt::unadopt(...)`. A method would interfere with methods of the same
    /// name on the contents of a `Rc` used through `Deref`.
    ///
    /// # Memory Leaks
    ///
    /// Failure to call this function when removing an owned `Rc` from `this`
    /// is safe, but may result in a memory leak.
    fn unadopt(this: &Self, other: &Self);
}

/// Implementation of [`Adopt`] for [`Rc`] which enables `Rc`s to form a cycle
/// of strong references that are reaped by `Rc`'s [`Drop`] implementation.
unsafe impl<T> Adopt for Rc<T> {
    /// `Rc`'s inner owned value. For an `Rc<T>`, the inner owned value is a
    /// `T`.
    type Inner = T;

    /// TODO: document me!
    fn adopt(this: &mut Self, other: &Self)
    where
        Self::Inner: Trace,
    {
        // Use internal iteration on `this`'s owned `Rc`s to look for `other`.
        //
        // If `this` can yield a mutable reference to an `Rc` that has the same
        // inner allocation as `other`, we can safely assert that `this` owns
        // an `Rc` with the same `RcBox` as `other`.
        let needle = other.inner() as *const _;
        let mut found = None;
        Trace::yield_owned_rcs(this.as_ref(), |node| {
            if found.is_some() {
                return;
            }
            // If `this` yields an `Rc` with the same `RcBox` as the given
            // `other`, `this` owns a `Rc` that points to the same allocation as
            // `other`, which fulfills the safety invariant of `adopt_unchecked`.
            if ptr::eq(needle, node.inner()) {
                // Clone the node we've found that matches the needle so we can
                // save a reference to the `RcBox` we want to adopt.
                found = Some(Rc::clone(node));
            }
        });
        if let Some(node) = found {
            // SAFETY: `yield_owned_rcs` yielded a mutable reference to an `Rc`
            // matching `other`'s inner allocation, which means `this` must own
            // an `Rc` matching `other`.
            //
            // This uphold's adopt_unchecked's safety invariant.
            unsafe {
                Self::adopt_unchecked(this, &node);
            }
        }
    }

    /// Perform bookkeeping to record that `this` has an owned reference to
    /// `other`.
    ///
    /// Adoption is a one-way link, or a directed edge in the object graph which
    /// means "`this` owns `other`".
    ///
    /// `adopt` can be called multiple times for a pair of `Rc`s. Each call to
    /// `adopt` indicates that `this` owns one distinct clone of `other`.
    ///
    /// This is an associated function that needs to be used as
    /// `Rc::adopt_unchecked(...)`. A method would interfere with methods of the same
    /// name on the contents of a `Rc` used through `Deref`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `this` owns a strong reference to `other`.
    ///
    /// Callers should call [`unadopt`] when `this` no longer holds a strong
    /// reference to `other` to avoid memory leaks, but this is not required for
    /// soundness.
    ///
    /// Calling `adopt` does not increment the strong count of `other`. Callers
    /// must ensure that `other` has been cloned and stored in the `T` contained
    /// by `this`.
    ///
    /// # Examples
    ///
    /// The following implements a self-referential array.
    ///
    /// ```rust
    /// use cactusref::{Adopt, Rc};
    /// use std::cell::RefCell;
    ///
    /// #[derive(Default)]
    /// struct Array {
    ///     buffer: Vec<Rc<RefCell<Self>>>,
    /// }
    ///
    /// let array = Rc::new(RefCell::new(Array::default()));
    /// for _ in 0..10 {
    ///     let item = Rc::clone(&array);
    ///     unsafe {
    ///         Rc::adopt_unchecked(&array, &item);
    ///     }
    ///     array.borrow_mut().buffer.push(item);
    /// }
    /// let weak = Rc::downgrade(&array);
    /// // 1 for the array binding, 10 for the `Rc`s in buffer
    /// assert_eq!(Rc::strong_count(&array), 11);
    /// drop(array);
    /// assert!(weak.upgrade().is_none());
    /// assert_eq!(weak.weak_count(), 0);
    /// ```
    ///
    /// [`unadopt`]: Rc::unadopt
    unsafe fn adopt_unchecked(this: &Self, other: &Self) {
        // Self-adoptions have no effect.
        if ptr::eq(this, other) {
            // Store a loopback reference to `other` in `this`. This bookkeeping
            // logs a strong reference and is used for discovering cycles.
            //
            // SAFETY: `this` is a live `Rc` so the `links` on its inner
            // allocation are an inhabited `MaybeUninit`.
            let mut links = this.inner().links().borrow_mut();
            links.insert(Link::loopback(other.ptr));
            return;
        }
        // Store a forward reference to `other` in `this`. This bookkeeping logs
        // a strong reference and is used for discovering cycles.
        //
        // SAFETY: `this` is a live `Rc` so the `links` on its inner allocation
        // are an inhabited `MaybeUninit`.
        let mut links = this.inner().links().borrow_mut();
        links.insert(Link::forward(other.ptr));
        // `this` and `other` may point to the same allocation. Drop the borrow
        // on `links` before accessing `other` to avoid a already borrowed error
        // from the `RefCell`.
        drop(links);
        // Store a backward reference to `this` in `other`. This bookkeeping is
        // used for discovering cycles.
        //
        // SAFETY: `this` is a live `Rc` so the `links` on its inner allocation
        // are an inhabited `MaybeUninit`.
        let mut links = other.inner().links().borrow_mut();
        links.insert(Link::backward(this.ptr));
    }

    /// Perform bookkeeping to record that `this` has removed an owned reference
    /// to `other`.
    ///
    /// Adoption is a one-way link, or a directed edge in the object graph which
    /// means "`this` owns `other`".
    ///
    /// This is an associated function that needs to be used as
    /// `Adopt::unadopt(...)`. A method would interfere with methods of the same
    /// name on the contents of a `Rc` used through `Deref`.
    ///
    /// # Memory Leaks
    ///
    /// Failure to call this function when removing an owned `Rc` from `this`
    /// is safe, but may result in a memory leak.
    ///
    /// # Examples
    ///
    /// The following implements a self-referential array.
    ///
    /// ```rust
    /// use cactusref::{Adopt, Rc};
    /// use std::cell::RefCell;
    ///
    /// #[derive(Default)]
    /// struct Array {
    ///     buffer: Vec<Rc<RefCell<Self>>>,
    /// }
    ///
    /// let array = Rc::new(RefCell::new(Array::default()));
    /// for _ in 0..10 {
    ///     let item = Rc::clone(&array);
    ///     unsafe {
    ///         Rc::adopt_unchecked(&array, &item);
    ///     }
    ///     array.borrow_mut().buffer.push(item);
    /// }
    /// let weak = Rc::downgrade(&array);
    /// // 1 for the array binding, 10 for the `Rc`s in buffer
    /// assert_eq!(Rc::strong_count(&array), 11);
    ///
    /// let head = array.borrow_mut().buffer.pop().unwrap();
    /// Rc::unadopt(&array, &head);
    ///
    /// drop(head);
    /// assert_eq!(Rc::strong_count(&array), 10);
    /// drop(array);
    /// assert!(weak.upgrade().is_none());
    /// assert_eq!(weak.weak_count(), 0);
    /// ```
    fn unadopt(this: &Self, other: &Self) {
        // Self-adoptions have no effect.
        if ptr::eq(this, other) {
            // Remove a loopback reference to `other` in `this`. This bookkeeping
            // logs a strong reference and is used for discovering cycles.
            //
            // SAFETY: `this` is a live `Rc` so the `links` on its inner
            // allocation are an inhabited `MaybeUninit`.
            let mut links = unsafe { this.inner().links().borrow_mut() };
            links.remove(Link::loopback(other.ptr), 1);
            return;
        }
        // Remove a forward reference to `other` in `this`. This bookkeeping
        // removes a strong reference and is used for discovering cycles.
        //
        // SAFETY: `this` is a live `Rc` so the `links` on its inner allocation
        // are an inhabited `MaybeUninit`.
        let mut links = unsafe { this.inner().links().borrow_mut() };
        links.remove(Link::forward(other.ptr), 1);
        // `this` and `other` may point to the same allocation. Drop the borrow
        // on `links` before accessing `other` to avoid a already borrowed error
        // from the `RefCell`.
        drop(links);
        // Remove a backward reference to `this` in `other`. This bookkeeping is
        // used for discovering cycles.
        //
        // SAFETY: `this` is a live `Rc` so the `links` on its inner allocation
        // are an inhabited `MaybeUninit`.
        let mut links = unsafe { other.inner().links().borrow_mut() };
        links.remove(Link::backward(this.ptr), 1);
    }
}
