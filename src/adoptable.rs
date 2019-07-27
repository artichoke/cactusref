use crate::link::Link;
use crate::ptr::RcBoxPtr;
use crate::Rc;

/// Perform bookkeeping to link two objects with an owned reference.
///
/// Calling [`Adoptable::adopt`] builds an object graph which can be used by
/// implementors to detect cycles.
///
/// **Warning**: this trait is unsafe because if it is implemented incorrectly,
/// memory may leak, double-free, or free too early and result in a dangling
/// pointer.
pub unsafe trait Adoptable {
    /// Perform bookkeeping to record that `this` has an owned reference to
    /// `other`. Adoption is a one-way link.
    ///
    /// **Warning**: this function unsafe because if it is used incorrectly,
    /// memory may leak, double-free, or free too early and result in a dangling
    /// pointer.
    unsafe fn adopt(this: &Self, other: &Self);

    /// Perform bookkeeping to record that `this` no longer has an owned
    /// reference to `other`. Adoption is a one-way link.
    ///
    /// **Warning**: this function unsafe because if it is used incorrectly,
    /// memory may leak, double-free, or free too early and result in a dangling
    /// pointer.
    unsafe fn unadopt(this: &Self, other: &Self);
}

/// Implementation of [`Adoptable`] for [`Rc`] which enables `Rc`s to form a
/// cycle of strong references that are reaped by `Rc`'s [`Drop`]
/// implementation.
unsafe impl<T: ?Sized> Adoptable for Rc<T> {
    /// Perform bookkeeping to record that `this` holds an owned reference to
    /// `other`.
    ///
    /// `Drop` expects that `this` currently holds a refernce to `other` by the
    /// time `adopt` is called, although it may be safe to call adopt before the
    /// reference is held.
    ///
    /// # ⚠️ Safety
    ///
    /// `CactusRef` relies on proper use of [`Adoptable::adopt`] and
    /// [`Adoptable::unadopt`] to maintain bookkeeping about the object graph
    /// for breaking cycles. These functions are unsafe because improperly
    /// managing the bookkeeping can cause the `Rc` `Drop` implementation to
    /// deallocate cycles while they are still externally reachable. All held
    /// `Rc`s that point to members of the now deallocated cycle will dangle.
    ///
    /// `CactusRef` makes a best-effort attempt to abort the program if an
    /// access to a dangling `Rc` occurs.
    ///
    /// # Examples
    ///
    /// The following implements a self-referential array.
    ///
    /// ```rust
    /// use cactusref::{Adoptable, Rc};
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
    ///     array.borrow_mut().buffer.push(item);
    ///     unsafe {
    ///         Rc::adopt(&array, &array);
    ///     }
    /// }
    /// let weak = Rc::downgrade(&array);
    /// // 1 for the array binding, 10 for the `Rc`s in buffer
    /// assert_eq!(Rc::strong_count(&array), 11);
    /// drop(array);
    /// assert!(weak.upgrade().is_none());
    /// assert_eq!(weak.weak_count(), Some(1));
    /// ```
    unsafe fn adopt(this: &Self, other: &Self) {
        // Adoption signals the intent to take an owned reference to `other`, so
        // always increment the strong count of other. This allows `this` to be
        // self-referential and allows `this` to own multiple references to
        // `other`. These behaviors allow implementing self-referential
        // collection types.

        // Store a forward reference to `other` in `this`. This bookkeeping logs
        // a strong reference and is used for discovering cycles.
        let mut links = this.inner().links.borrow_mut();
        links.insert(Link::forward(other.ptr));
        // `this` and `other` may be the same `Rc`. Drop the borrow on `links`
        // before accessing `other` to avoid a already borrowed error from the
        // `RefCell`.
        drop(links);
        // Store a backward reference to `this` in `other`. This bookkeeping is
        // used for discovering cycles.
        let mut links = other.inner().links.borrow_mut();
        links.insert(Link::backward(this.ptr));
    }

    /// Perform bookkeeping to record that `this` no longer holds an owned
    /// reference to `other`.
    ///
    /// `Drop` expects that `this` currently does not hold a refernce to `other`
    /// by the time `unadopt` is called, although it may be safe to call adopt
    /// before the reference is held.
    ///
    /// # ⚠️ Safety
    ///
    /// `CactusRef` relies on proper use of [`Adoptable::adopt`] and
    /// [`Adoptable::unadopt`] to maintain bookkeeping about the object graph
    /// for breaking cycles. These functions are unsafe because improperly
    /// managing the bookkeeping can cause the `Rc` `Drop` implementation to
    /// deallocate cycles while they are still externally reachable. All held
    /// `Rc`s that point to members of the now deallocated cycle will dangle.
    ///
    /// `CactusRef` makes a best-effort attempt to abort the program if an
    /// access to a dangling `Rc` occurs.
    ///
    /// # Examples
    ///
    /// The following implements a self-referential array.
    ///
    /// ```rust
    /// use cactusref::{Adoptable, Rc};
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
    ///     array.borrow_mut().buffer.push(item);
    ///     unsafe {
    ///         Rc::adopt(&array, &array);
    ///     }
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
    /// assert_eq!(weak.weak_count(), Some(1));
    /// ```
    unsafe fn unadopt(this: &Self, other: &Self) {
        // Remove a forward reference to `other` in `this`. This bookkeeping
        // logs a strong reference and is used for discovering cycles.
        let mut links = this.inner().links.borrow_mut();
        links.remove(Link::forward(other.ptr), 1);
        // `this` and `other` may be the same `Rc`. Drop the borrow on `links`
        // before accessing `other` to avoid a already borrowed error from the
        // `RefCell`.
        drop(links);
        // Remove a backward reference to `this` in `other`. This bookkeeping is
        // used for discovering cycles.
        let mut links = other.inner().links.borrow_mut();
        links.remove(Link::backward(this.ptr), 1);
    }
}
