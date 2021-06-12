use core::cell::Cell;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ptr::{self, NonNull};
use hashbrown::{hash_map, HashMap};

use crate::rc::{RcBox, RcInnerPtr};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Kind {
    Forward,
    Backward,
}

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
    pub fn insert(&mut self, other: Link<T>) {
        *self.registry.entry(other).or_insert(0) += 1;
    }

    #[inline]
    pub fn remove(&mut self, other: Link<T>, strong: usize) {
        match self.registry.get(&other).copied().unwrap_or_default() {
            count if count <= strong => self.registry.remove(&other),
            count => self.registry.insert(other, count - strong),
        };
    }

    #[inline]
    pub fn clear(&mut self) {
        self.registry.clear()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> hash_map::Iter<Link<T>, usize> {
        self.registry.iter()
    }

    #[inline]
    pub fn drain_filter<F>(&mut self, f: F) -> hash_map::DrainFilter<Link<T>, usize, F>
    where
        F: FnMut(&Link<T>, &mut usize) -> bool,
    {
        self.registry.drain_filter(f)
    }
}

impl<T> Clone for Links<T> {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
        }
    }
}

impl<T> Default for Links<T> {
    fn default() -> Self {
        Self {
            registry: HashMap::default(),
        }
    }
}

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

impl<T> Link<T> {
    #[inline]
    pub fn forward(ptr: NonNull<RcBox<T>>) -> Self {
        Self {
            ptr,
            kind: Kind::Forward,
        }
    }

    #[inline]
    pub fn backward(ptr: NonNull<RcBox<T>>) -> Self {
        Self {
            ptr,
            kind: Kind::Backward,
        }
    }

    #[inline]
    pub fn link_kind(&self) -> Kind {
        self.kind
    }

    #[inline]
    pub fn as_forward(&self) -> Self {
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
    fn weak_ref(&self) -> &Cell<usize> {
        unsafe { self.ptr.as_ref().weak_ref() }
    }

    fn strong_ref(&self) -> &Cell<usize> {
        unsafe { self.ptr.as_ref().strong_ref() }
    }
}

impl<T> Copy for Link<T> {}

impl<T> Clone for Link<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            kind: self.kind,
        }
    }
}

impl<T> PartialEq for Link<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && ptr::eq(self.as_ptr(), other.as_ptr())
    }
}

impl<T> Eq for Link<T> {}

impl<T> Hash for Link<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
        self.kind.hash(state);
    }
}
