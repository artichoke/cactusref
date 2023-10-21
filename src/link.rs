#![allow(clippy::iter_without_into_iter)]

//! This module includes data structures for building an object graph.

use core::cell::Cell;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::num::NonZeroUsize;
use core::ptr::{self, NonNull};

use crate::hash::hash_map::{ExtractIf, Iter};
use crate::hash::HashMap;
use crate::rc::{RcBox, RcInnerPtr};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum Kind {
    Forward,
    Backward,
    Loopback,
}

/// A collection of forward and backward links and their corresponding adoptions.
pub(crate) struct Links<T> {
    registry: HashMap<Link<T>, usize>,
}

impl<T> fmt::Debug for Links<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Links")
            .field("registry", &self.registry)
            .finish()
    }
}

impl<T> Links<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            registry: HashMap::default(),
        }
    }

    #[inline]
    pub fn insert(&mut self, other: Link<T>) {
        *self.registry.entry(other).or_insert(0) += 1;
    }

    #[inline]
    pub fn remove(&mut self, other: Link<T>, strong: usize) {
        let count = self.registry.get(&other).copied().unwrap_or_default();
        let remaining_strong_count = count.checked_sub(strong).and_then(NonZeroUsize::new);
        if let Some(remaining_strong_count) = remaining_strong_count {
            self.registry.insert(other, remaining_strong_count.get());
        } else {
            self.registry.remove(&other);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.registry.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Link<T>, usize> {
        self.registry.iter()
    }

    #[inline]
    pub fn extract_if<F>(&mut self, f: F) -> ExtractIf<'_, Link<T>, usize, F>
    where
        F: FnMut(&Link<T>, &mut usize) -> bool,
    {
        self.registry.extract_if(f)
    }
}

/// Link represents a directed edge in the object graph of strong CactusRef `Rc`
/// smart pointers.
///
/// Links can be one of several types:
///
/// - Forward, which means the `Rc` storing the link is adopting the link's
///   pointee.
/// - Backward, which means this `Rc` is being adopted by the link's pointee.
/// - Loopback, which means the `Rc` has adopted itself.
pub(crate) struct Link<T> {
    ptr: NonNull<RcBox<T>>,
    kind: Kind,
}

impl<T> fmt::Debug for Link<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Link")
            .field("ptr", &self.ptr)
            .field("kind", &self.kind)
            .finish()
    }
}

impl<T> fmt::Pointer for Link<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr.as_ptr(), f)
    }
}

impl<T> Link<T> {
    #[inline]
    pub const fn forward(ptr: NonNull<RcBox<T>>) -> Self {
        Self {
            ptr,
            kind: Kind::Forward,
        }
    }

    #[inline]
    pub const fn backward(ptr: NonNull<RcBox<T>>) -> Self {
        Self {
            ptr,
            kind: Kind::Backward,
        }
    }

    #[inline]
    pub const fn loopback(ptr: NonNull<RcBox<T>>) -> Self {
        Self {
            ptr,
            kind: Kind::Loopback,
        }
    }

    #[inline]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    #[inline]
    pub const fn as_forward(&self) -> Self {
        Self::forward(self.ptr)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut RcBox<T> {
        self.ptr.as_ptr()
    }

    #[inline]
    pub fn as_ref(&self) -> &RcBox<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub fn into_raw_non_null(self) -> NonNull<RcBox<T>> {
        self.ptr
    }
}

impl<T> RcInnerPtr for Link<T> {
    #[inline(always)]
    fn weak_ref(&self) -> &Cell<usize> {
        unsafe { self.ptr.as_ref().weak_ref() }
    }

    #[inline(always)]
    fn strong_ref(&self) -> &Cell<usize> {
        unsafe { self.ptr.as_ref().strong_ref() }
    }
}

impl<T> Clone for Link<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Link<T> {}

impl<T> PartialEq for Link<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && ptr::eq(self.as_ptr(), other.as_ptr())
    }
}

impl<T> Eq for Link<T> {}

impl<T> Hash for Link<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
        self.kind.hash(state);
    }
}
