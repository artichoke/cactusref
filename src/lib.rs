#![feature(
    allocator_api,
    core_intrinsics,
    dropck_eyepatch,
    set_ptr_value,
    slice_ptr_get
)]
#![allow(incomplete_features)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::inline_always)]
#![allow(clippy::option_if_let_else)]
#![allow(unknown_lints)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]

//! Single-threaded, cycle-aware, reference-counting pointers. 'Rc' stands
//! for 'Reference Counted'.
//!
//! The type [`Rc<T>`] provides shared ownership of a value of type `T`,
//! allocated in the heap. Invoking [`clone`] on [`Rc`] produces a new pointer
//! to the same value in the heap. When the last externally reachable [`Rc`]
//! pointer to a given value is destroyed, the pointed-to value is also
//! destroyed.
//!
//! [`Rc<T>`]: crate::Rc
//! [`clone`]: Clone::clone
//!
//! `Rc` can **detect and deallocate cycles** of `Rc`s through the use of
//! [`Adopt`]. Cycle detection is a zero-cost abstraction.
//!
//! # Nightly
//!
//! CactusRef depends on several unstable Rust features and can only be built
//! on a nightly toolchain. CactusRef reimplements several compiler internals
//! from [alloc], which means it is only safe to build CactusRef with the same
//! nightly compiler as the one pinned in its `rust-toolchain` file.
//!
//! [alloc]: https://doc.rust-lang.org/stable/alloc/
//!
//! # CactusRef vs. `std::rc`
//!
//! The `Rc` in CactusRef is derived from [`std::rc::Rc`] and CactusRef
//! implements most of the API from `std`.
//!
//! CactusRef does not implement the following APIs that are present on
//! [`std::rc::Rc`]:
//!
//! - [`std::rc::Rc::downcast`](std::rc::Rc::downcast)
//! - [`CoerceUnsized`](core::ops::CoerceUnsized)
//! - [`DispatchFromDyn`](core::ops::DispatchFromDyn)
//! - `From<Cow<'_, T>>`
//!
//! CactusRef cannot be used with unsized types like `[T]` or `str`.
//!
//! If you do not depend on these APIs, CactusRef is a drop-in replacement for
//! [`std::rc::Rc`].
//!
//! Like [`std::rc`], [`Rc`] and [`Weak`] are not `Send` and are not `Sync`.
//!
//! [`std::rc`]: https://doc.rust-lang.org/stable/std/rc/index.html
//!
//! # Building an object graph
//!
//! CactusRef smart pointers can be used to implement a tracing garbage
//! collector local to a graph objects. Graphs of CactusRefs are cycle-aware and
//! can deallocate a cycle of strong references that is otherwise unreachable
//! from the rest of the object graph, unlike [`std::rc::Rc`].
//!
//! `CactusRef` relies on proper use of [`Adopt::adopt`] and [`Adopt::unadopt`]
//! to maintain bookkeeping about the object graph for breaking cycles. These
//! functions are unsafe because improperly managing the bookkeeping can cause
//! the `Rc` drop implementation to deallocate cycles while they are still
//! externally reachable. Failure to uphold [`Adopt`]'s safety invariants will
//! result in *[undefined behavior]* and held `Rc`s that point to members of the
//! now deallocated cycle may dangle.
//!
//! [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
//!
//! CactusRef makes a best-effort attempt to abort the program if it detects an
//! access to a dangling `Rc`.
//!
//! # Cycle Detection
//!
//! `Rc` implements [`Adopt`] to log bookkeeping entries for strong ownership
//! links to other `Rc`s that may form a cycle. The ownership links tracked by
//! these bookkeeping entries form an object graph of reachable `Rc`s. On
//! `drop`, `Rc` uses these entries to conduct a reachability trace of the
//! object graph to determine if it is part of an _orphaned cycle_. An orphaned
//! cycle is a cycle where the only strong references to all nodes in the cycle
//! come from other nodes in the cycle.
//!
//! Cycle detection is a zero-cost abstraction. If you never
//! `use cactusref::Adopt;`, `drop` uses the same implementation as
//! [`std::rc::Rc`] (and leaks in the same way as `std::rc::Rc` if you form a
//! cycle of strong references). The only costs you pay are the memory costs of
//! one empty hash map used to track adoptions and an if statement to check if
//! these structures are empty on `drop`.
//!
//! Cycle detection uses breadth-first search for traversing the object graph.
//! The algorithm supports arbitrarily large object graphs and will not overflow
//! the stack during the reachability trace.
//!
//! [`std::rc::Rc`]: https://doc.rust-lang.org/stable/std/rc/struct.Rc.html

#![doc(html_root_url = "https://docs.rs/cactusref/0.1.0")]

// Ensure code blocks in README.md compile
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

extern crate alloc;
#[macro_use]
extern crate log;

mod adopt;
mod cycle;
mod drop;
mod hash;
mod link;
mod rc;

// Doc modules
#[cfg(any(doctest, docsrs))]
#[path = "doc/implementing_self_referential_data_structures.rs"]
/// Examples of implementing self-referential data structures with CactusRef.
pub mod implementing_self_referential_data_structures;

pub use adopt::Adopt;
pub use rc::Rc;
pub use rc::Weak;

/// Cactus alias for [`Rc`].
pub type CactusRef<T> = Rc<T>;

/// Cactus alias for [`Weak`].
pub type CactusWeakRef<T> = Weak<T>;
