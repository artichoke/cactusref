//! `CactusRef` can be used to implement collections that own strong references
//! to themselves.
//!
//! # Doubly-linked List
//!
//! The following implements a doubly-linked list that is fully deallocated once
//! the `list` binding is dropped.
//!
//! ```rust
//! use std::cell::RefCell;
//! use std::iter;
//!
//! use cactusref::{Adopt, Rc};
//!
//! struct Node<T> {
//!     pub prev: Option<Rc<RefCell<Self>>>,
//!     pub next: Option<Rc<RefCell<Self>>>,
//!     pub data: T,
//! }
//!
//! struct List<T> {
//!     pub head: Option<Rc<RefCell<Node<T>>>>,
//! }
//!
//! impl<T> List<T> {
//!     fn pop(&mut self) -> Option<Rc<RefCell<Node<T>>>> {
//!         let head = self.head.take()?;
//!         let tail = head.borrow_mut().prev.take();
//!         let next = head.borrow_mut().next.take();
//!         if let Some(ref tail) = tail {
//!             unsafe {
//!                 Rc::unadopt(&head, tail);
//!                 Rc::unadopt(tail, &head);
//!             }
//!             tail.borrow_mut().next = next.as_ref().map(Rc::clone);
//!             if let Some(ref next) = next {
//!                 unsafe {
//!                     Rc::adopt_unchecked(tail, next);
//!                 }
//!             }
//!         }
//!         if let Some(ref next) = next {
//!             unsafe {
//!                 Rc::unadopt(&head, next);
//!                 Rc::unadopt(next, &head);
//!             }
//!             next.borrow_mut().prev = tail.as_ref().map(Rc::clone);
//!             if let Some(ref tail) = tail {
//!                 unsafe {
//!                     Rc::adopt_unchecked(next, tail);
//!                 }
//!             }
//!         }
//!         self.head = next;
//!         Some(head)
//!     }
//! }
//!
//! impl<T> From<Vec<T>> for List<T> {
//!     fn from(list: Vec<T>) -> Self {
//!         let nodes = list
//!             .into_iter()
//!             .map(|data| {
//!                 Rc::new(RefCell::new(Node {
//!                     prev: None,
//!                     next: None,
//!                     data,
//!                 }))
//!             })
//!             .collect::<Vec<_>>();
//!         for i in 0..nodes.len() - 1 {
//!             let curr = &nodes[i];
//!             let next = &nodes[i + 1];
//!             curr.borrow_mut().next = Some(Rc::clone(next));
//!             next.borrow_mut().prev = Some(Rc::clone(curr));
//!             unsafe {
//!                 Rc::adopt_unchecked(curr, next);
//!                 Rc::adopt_unchecked(next, curr);
//!             }
//!         }
//!         let tail = &nodes[nodes.len() - 1];
//!         let head = &nodes[0];
//!         tail.borrow_mut().next = Some(Rc::clone(head));
//!         head.borrow_mut().prev = Some(Rc::clone(tail));
//!         unsafe {
//!             Rc::adopt_unchecked(tail, head);
//!             Rc::adopt_unchecked(head, tail);
//!         }
//!
//!         let head = Rc::clone(head);
//!         Self { head: Some(head) }
//!     }
//! }
//!
//! let list = iter::repeat(())
//!     .map(|_| "a".repeat(1024 * 1024))
//!     .take(10)
//!     .collect::<Vec<_>>();
//! let mut list = List::from(list);
//!
//! let head = list.pop().unwrap();
//! assert_eq!(Rc::strong_count(&head), 1);
//! assert!(head.borrow().data.starts_with('a'));
//!
//! // The new head of the list is owned three times:
//! //
//! // - itself.
//! // - the `prev` pointer to it from it's next element.
//! // - the `next` pointer from the list's tail.
//! assert_eq!(list.head.as_ref().map(Rc::strong_count), Some(3));
//!
//! // The popped head is no longer part of the graph and can be safely dropped
//! // and deallocated.
//! let weak = Rc::downgrade(&head);
//! drop(head);
//! assert!(weak.upgrade().is_none());
//!
//! drop(list);
//! // all memory consumed by the list nodes is reclaimed.
//! ```
