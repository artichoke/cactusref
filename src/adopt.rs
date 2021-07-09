use alloc::boxed::Box;
use core::ptr::{self, NonNull};

use crate::graph::Graph;
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
/// - Dangling `Rc`s which will cause a use after free.
///
/// [`adopt_unchecked`]: Adopt::adopt_unchecked
/// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
pub unsafe trait Adopt: sealed::Sealed {
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
        if ptr::eq(this, other) {
            return;
        }
        std::dbg!();
        match (this.inner().graph.get(), other.inner().graph.get()) {
            (Some(mut left), Some(right)) if left == right => {
                (*left.as_mut()).link(this.ptr, other.ptr);
            }
            (Some(mut left), Some(right)) => {
                let right = Box::from_raw(right.as_ptr());
                (*left.as_mut()).merge(right);
                (*left.as_mut()).link(this.ptr, other.ptr);
            }
            (None, Some(mut right)) => {
                this.inner().graph.set(Some(right));
                (*right.as_mut()).link(this.ptr, other.ptr);
            }
            (Some(mut left), None) => {
                other.inner().graph.set(Some(left));
                (*left.as_mut()).link(this.ptr, other.ptr);
            }
            (None, None) => {
                let mut graph = Graph::new();
                graph.link(this.ptr, other.ptr);
                let graph = Box::new(graph);
                let graph = Box::into_raw(graph);
                let graph = NonNull::new_unchecked(graph);
                this.inner().graph.set(Some(graph));
                other.inner().graph.set(Some(graph));
            }
        }
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
        std::dbg!();
        if let Some(mut graph) = this.inner().graph.get() {
            std::dbg!(unsafe { &(*graph.as_mut()) });
            if let Some(split) = unsafe { (*graph.as_mut()).try_split_off(this.ptr, other.ptr) } {
                let split = std::dbg!(split);
                if split.is_empty() {
                    other.inner().graph.set(None);
                } else {
                    let split = Box::into_raw(split);
                    // SAFETY: pointers obtained from `Box::into_raw` are always
                    // non-null.
                    let split = unsafe { NonNull::new_unchecked(split) };
                    other.inner().graph.set(Some(split));
                }
            } else {
                std::dbg!((this.ptr, other.ptr));
                unsafe {
                    (*graph.as_mut()).unlink(this.ptr, other.ptr);
                }
            }
            std::dbg!(unsafe { &(*graph.as_mut()) });
            if unsafe { (*graph.as_ptr()).is_empty() } {
                std::dbg!();
                let _graph = unsafe { Box::from_raw(graph.as_ptr()) };
                this.inner().graph.set(None);
                other.inner().graph.set(None);
            }
        }
    }
}
