#![feature(
    allocator_api,
    alloc_layout_extra,
    box_into_raw_non_null,
    core_intrinsics,
    dropck_eyepatch,
    optin_builtin_traits,
    ptr_internals
)]
#![deny(warnings, intra_doc_link_resolution_failure)]
#![deny(clippy::all, clippy::pedantic)]

// does not support Rc::downcast
// Does not support operations on Rc<[T]>

#[macro_use]
extern crate log;

mod link;
mod ptr;
mod rc;
mod reachable;
#[cfg(test)]
mod tests;
mod weak;

pub use rc::{Rc, Rc as CactusRef};
pub use reachable::Reachable;
pub use weak::{Weak, Weak as CactusWeakRef};
