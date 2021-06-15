use alloc::vec;

use crate::hash::{HashMap, HashSet};
use crate::link::{Kind, Link};
use crate::rc::RcInnerPtr;
use crate::Rc;

impl<T> Rc<T> {
    /// Traverse the linked object graph from the given `Rc` to determine if the
    /// graph is not externally reachable.
    ///
    /// Cycles are discovered using breadth-first search of the graph's adopted
    /// links.
    ///
    /// If this function returns `Some(_)`, the graph of `Rc`s would leak using
    /// `std::rc::Rc`.
    ///
    /// This funtion returns a hash map of forward links to the number of times
    /// the link appears in the cycle.
    ///
    /// This function is invoked during `drop` to determine which strategy to use
    /// for deallocating a group of `Rc`s.
    pub(crate) fn orphaned_cycle(this: &Self) -> Option<HashMap<Link<T>, usize>> {
        let cycle = cycle_refs(Link::forward(this.ptr));
        if cycle.is_empty() {
            return None;
        }
        let has_external_owners = cycle
            .iter()
            .any(|(item, &cycle_owned_refs)| item.strong() > cycle_owned_refs);
        if has_external_owners {
            None
        } else {
            Some(cycle)
        }
    }
}

// Perform a breadth first search over all of the forward and backward links to
// determine the clique of nodes in a cycle and their strong counts.
fn cycle_refs<T>(this: Link<T>) -> HashMap<Link<T>, usize> {
    // These collections track compute the layout of the object graph in linear
    // time in the size of the graph.
    let mut cycle_owned_refs = HashMap::default();
    let mut discovered = vec![this];
    let mut visited = HashSet::default();

    // crawl the graph
    while let Some(node) = discovered.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);

        let links = unsafe { node.as_ref().links().borrow() };
        for (&link, &strong) in links.iter() {
            if let Kind::Forward | Kind::Loopback = link.kind() {
                cycle_owned_refs
                    .entry(link)
                    .and_modify(|count| *count += strong)
                    .or_insert(strong);
                discovered.push(link);
            } else {
                cycle_owned_refs.entry(link.as_forward()).or_default();
            }
        }
    }

    #[cfg(debug_assertions)]
    debug_cycle(&cycle_owned_refs);
    cycle_owned_refs
}

#[cfg(debug_assertions)]
fn debug_cycle<T>(cycle: &HashMap<Link<T>, usize>) {
    use alloc::vec::Vec;

    if cycle.is_empty() {
        trace!("cactusref reachability test found no cycles");
        return;
    }

    let counts = cycle
        .iter()
        .map(|(item, cycle_count)| (item.as_ref().strong(), cycle_count))
        .collect::<Vec<_>>();
    let has_external_owners = cycle
        .iter()
        .any(|(item, &cycle_owned_refs)| item.strong() > cycle_owned_refs);

    if has_external_owners {
        trace!(
            "cactusref reachability test found externally owned cycle with (strong, cycle) counts: {:?}",
            counts
        );
    } else {
        trace!(
            "cactusref reachability test found unreachable cycle  with (strong, cycle) counts: {:?}",
            counts
        );
    }
}
