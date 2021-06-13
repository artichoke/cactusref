use core::ptr;

use crate::link::Link;
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
/// Calling [`adopt`] builds an object graph which can be used by to detect
/// cycles.
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
/// [`adopt`]: Adopt::adopt
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
    /// `Adopt::adopt(...)`. A method would interfere with methods of the same
    /// name on the contents of a `Rc` used through `Deref`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `this` owns a strong reference to `other`.
    ///
    /// Callers must call [`unadopt`] when `this` no longer holds a strong
    /// reference to `other`.
    ///
    /// [`unadopt`]: Adopt::unadopt
    unsafe fn adopt(this: &Self, other: &Self);

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
    /// # Safety
    ///
    /// Callers must ensure that `this` has removed an owned reference to
    /// `other`.
    ///
    /// For each call to `Adopt::unadopt(&this, &other)`, callers must ensure
    /// that a matching call was made to `Adopt::adopt(&this, &other)`.
    unsafe fn unadopt(this: &Self, other: &Self);
}

/// Implementation of [`Adoptable`] for [`Rc`] which enables `Rc`s to form a
/// cycle of strong references that are reaped by `Rc`'s [`Drop`]
/// implementation.
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
    /// `Rc::adopt(...)`. A method would interfere with methods of the same
    /// name on the contents of a `Rc` used through `Deref`.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `this` owns a strong reference to `other`.
    ///
    /// Callers must call [`unadopt`] when `this` no longer holds a strong
    /// reference to `other`.
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
    ///         Rc::adopt(&array, &item);
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
    unsafe fn adopt(this: &Self, other: &Self) {
        // Self-adoptions have no effect.
        if ptr::eq(this, other) {
            // Store a forward reference to `other` in `this`. This bookkeeping logs
            // a strong reference and is used for discovering cycles.
            let mut links = this.inner().links().borrow_mut();
            links.insert(Link::loopback(other.ptr));
            return;
        }
        // Store a forward reference to `other` in `this`. This bookkeeping logs
        // a strong reference and is used for discovering cycles.
        let mut links = this.inner().links().borrow_mut();
        links.insert(Link::forward(other.ptr));
        // `this` and `other` may be the same `Rc`. Drop the borrow on `links`
        // before accessing `other` to avoid a already borrowed error from the
        // `RefCell`.
        drop(links);
        // Store a backward reference to `this` in `other`. This bookkeeping is
        // used for discovering cycles.
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
    /// # Safety
    ///
    /// Callers must ensure that `this` has removed an owned reference to
    /// `other`.
    ///
    /// For each call to `Adopt::unadopt(&this, &other)`, callers must ensure
    /// that a matching call was made to `Adopt::adopt(&this, &other)`.
    ///
    /// This crate makes a best-effort attempt to abort the program if an access
    /// to a dangling `Rc` occurs.
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
    ///         Rc::adopt(&array, &item);
    ///     }
    ///     array.borrow_mut().buffer.push(item);
    /// }
    /// let weak = Rc::downgrade(&array);
    /// // 1 for the array binding, 10 for the `Rc`s in buffer
    /// assert_eq!(Rc::strong_count(&array), 11);
    /// let head = array.borrow_mut().buffer.pop().unwrap();
    /// unsafe {
    ///     Rc::unadopt(&array, &head);
    /// }
    /// drop(head);
    /// assert_eq!(Rc::strong_count(&array), 10);
    /// drop(array);
    /// assert!(weak.upgrade().is_none());
    /// assert_eq!(weak.weak_count(), 0);
    /// ```
    unsafe fn unadopt(this: &Self, other: &Self) {
        // Remove a forward reference to `other` in `this`. This bookkeeping
        // removes a strong reference and is used for discovering cycles.
        let mut links = this.inner().links().borrow_mut();
        links.remove(Link::forward(other.ptr), 1);
        // `this` and `other` may be the same `Rc`. Drop the borrow on `links`
        // before accessing `other` to avoid a already borrowed error from the
        // `RefCell`.
        drop(links);
        // Remove a backward reference to `this` in `other`. This bookkeeping is
        // used for discovering cycles.
        let mut links = other.inner().links().borrow_mut();
        links.remove(Link::backward(this.ptr), 1);
    }
}
