use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use core::mem;
use core::ptr::NonNull;

use crate::hash::HashSet;
use crate::rc::{RcBox, RcInnerPtr};

struct Source<T> {
    inner: NonNull<RcBox<T>>,
}

impl<T> Clone for Source<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl<T> Copy for Source<T> {}

impl<T> PartialEq for Source<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Source<T> {}

impl<T> fmt::Debug for Source<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<T> fmt::Pointer for Source<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, f)
    }
}

impl<T> Source<T> {
    #[inline]
    const fn new(inner: NonNull<RcBox<T>>) -> Self {
        Self { inner }
    }

    #[inline]
    fn as_ptr(&self) -> *const RcBox<T> {
        self.inner.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&self) -> *mut RcBox<T> {
        self.inner.as_ptr()
    }
}

struct Destination<T> {
    inner: NonNull<RcBox<T>>,
}

impl<T> Clone for Destination<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl<T> Copy for Destination<T> {}

impl<T> PartialEq for Destination<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for Destination<T> {}

impl<T> fmt::Debug for Destination<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<T> fmt::Pointer for Destination<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.inner, f)
    }
}

impl<T> Destination<T> {
    #[inline]
    const fn new(inner: NonNull<RcBox<T>>) -> Self {
        Self { inner }
    }

    #[inline]
    fn as_ptr(&self) -> *const RcBox<T> {
        self.inner.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&self) -> *mut RcBox<T> {
        self.inner.as_ptr()
    }
}

pub(crate) struct Graph<T> {
    edges: Vec<(Source<T>, Destination<T>)>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn link(&mut self, source: NonNull<RcBox<T>>, destination: NonNull<RcBox<T>>) {
        self.edges
            .push((Source::new(source), Destination::new(destination)));
    }

    pub fn unlink(&mut self, source: NonNull<RcBox<T>>, destination: NonNull<RcBox<T>>) {
        let edge = (Source::new(source), Destination::new(destination));
        let index = self
            .edges
            .iter()
            .enumerate()
            .find(|(_, &elem)| elem == edge);
        if let Some((index, _)) = index {
            self.edges.swap_remove(index);
        }
    }

    pub fn num_links_between(
        &self,
        source: NonNull<RcBox<T>>,
        destination: NonNull<RcBox<T>>,
    ) -> usize {
        let edge = (Source::new(source), Destination::new(destination));
        self.edges.iter().filter(|&&elem| elem == edge).count()
    }

    pub fn merge(&mut self, other: Self) {
        let this_g_raw = if let Some(first) = self.edges.first() {
            // SAFETY: all nodes in a graph are reachable and not deallocated.
            unsafe { (*first.0.as_ptr()).graph }
        } else {
            panic!("attempted to merge into an empty graph");
        };

        for (left, right) in &other.edges {
            // SAFETY: all RcBox's in `other` point to `other`'s raw pointer.
            // This loop ensures these pointers will not dangle and point to
            // `self`'s raw pointer.
            //
            // SAFETY: all nodes in a graph are reachable and not deallocated.
            unsafe {
                (*left.as_mut_ptr()).graph = this_g_raw;
                (*right.as_mut_ptr()).graph = this_g_raw;
            }
        }

        self.edges.extend_from_slice(&other.edges);
    }

    pub fn try_split_off(
        &mut self,
        source: NonNull<RcBox<T>>,
        destination: NonNull<RcBox<T>>,
    ) -> Option<Box<Self>> {
        let edge = (Source::new(source), Destination::new(destination));
        let edge_index = self
            .edges
            .iter()
            .enumerate()
            .find(|(_, &elem)| elem == edge)
            .map(|(pos, _)| pos);
        let edge_index = if let Some(pos) = edge_index {
            match self.num_links_between(source, destination) {
                1 => {}
                n => return None,
            };
            pos
        } else {
            return None;
        };
        if self.num_links_between(destination, source) > 0 {
            return None;
        }
        // NOTE: `self.edges` is guaranteed to be non-empty here.
        debug_assert!(!self.edges.is_empty());

        let (left, right) = self.edges.swap_remove(edge_index);
        let mut graph = mem::replace(&mut self.edges, Vec::new());

        let mut right_nodes = HashSet::default();

        let mut discover_right = Vec::with_capacity(2 * graph.len());
        let mut right_graph = Vec::with_capacity(graph.len() - 1);
        discover_right.push(right.as_mut_ptr());

        while let Some(elem) = discover_right.pop() {
            if right_nodes.contains(&elem) {
                continue;
            }
            right_nodes.insert(elem);
            let mut edges = graph
                .drain_filter(|edge| edge.0.as_mut_ptr() == elem || edge.1.as_mut_ptr() == elem);
            for edge in edges {
                discover_right.push(edge.0.as_mut_ptr());
                discover_right.push(edge.1.as_mut_ptr());
                right_graph.push(edge);
            }
        }
        let new_g = Box::new(Self { edges: Vec::new() });
        let new_g_raw = Box::into_raw(new_g);
        for edge in &right_graph {
            unsafe {
                // SAFETY: all RcBox's in `right_graph` point to `self`'s raw
                // pointer.  This loop ensures these pointers will not dangle
                // and point to `new_g`'s raw pointer.
                //
                // SAFETY: all nodes in a graph are reachable and not
                // deallocated.
                (*edge.0.as_mut_ptr()).graph = new_g_raw;
                (*edge.1.as_mut_ptr()).graph = new_g_raw;
            }
        }
        // SAFETY: we previously obtained this pointer with `Box::into_raw` and
        // have not deallocated the `Box` or modified its contents.
        unsafe { Some(Box::from_raw(new_g_raw)) }
    }

    pub fn count_directed_edges_toward(&self, destination: NonNull<RcBox<T>>) -> usize {
        let destination = Destination::new(destination);
        self.edges
            .iter()
            .filter(|&&(_, dest)| dest == destination)
            .count()
    }

    pub fn is_externally_reachable(&self) -> bool {
        let mut visited_nodes = HashSet::default();
        let mut stack = Vec::with_capacity(self.edges.len() * 2);
        let mut iter = self.edges.iter();

        for &(left, right) in iter {
            stack.push(left.inner);
            stack.push(right.inner);
            while let Some(node) = stack.pop() {
                if visited_nodes.contains(&node) {
                    continue;
                }
                visited_nodes.insert(node);
                // SAFETY: RcBox's in a graph are live allocations.
                let strong = unsafe { (*node.as_ptr()).strong() };
                let graph_internal_strong = self.count_directed_edges_toward(node);
                if strong > graph_internal_strong {
                    return true;
                }
            }
        }
        false
    }
}
