---
title: "Cactus Harvesting: Cycle-Aware Reference Counting in Rust"
slug: cactus-harvesting
summary:
  "🌵 CactusRef lets you build cyclic data structures using strong references
  and knows how to deallocate unreachable cycles. You can use CactusRef to
  implement a doubly linked list. The CactusRef API is compatible with std::rc."
---

_This is a copy of a [blog post](https://hyperbo.la/w/cactus-harvesting/) by
[@lopopolo](https://github.com/lopopolo) which is about CactusRef as of
[`252ded0c`](https://github.com/artichoke/cactusref/tree/252ded0caf9bd9c5814cd8020b44176a1edbeb9e).
There was some
[discussion on Reddit](https://www.reddit.com/r/rust/comments/cdk731/cactus_harvesting_cycleaware_reference_counting/)._

🌵 CactusRef is a single-threaded, cycle-aware, reference counting smart pointer
[[docs](https://lopopolo.github.io/ferrocarril/cactusref/index.html)]
[[code](https://github.com/lopopolo/ferrocarril/tree/0052dc1d0b234c2535b8dd87a096e048bdc0819e/cactusref)].
CactusRef is nearly a drop-in replacement for
[`std::rc`](https://doc.rust-lang.org/std/rc/index.html)[^std-rc-api-compat]
from the Rust standard library. Throughout this post, `Rc` refers to
`cactusref::Rc`. I will refer to `std::rc::Rc` with its fully qualified name.

### Motivation

Building cyclic data structures in Rust is
[hard](https://news.ycombinator.com/item?id=16443688). When a `T` needs to have
multiple owners, it can be wrapped in a
[`std::rc::Rc`](https://doc.rust-lang.org/std/rc/index.html). `std::rc::Rc`,
however, is not cycle-aware. Creating a cycle of `std::rc::Rc`s will leak
memory. To work around this, an `std::rc::Rc` can be
[downgraded](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.downgrade)
into a [`std::rc::Weak`](https://doc.rust-lang.org/std/rc/struct.Weak.html).

### `std::rc::Rc` Limitations

Strong references are much more convenient to work with than weak references.
Imagine the following code (written in Ruby) to create a ring:

```ruby
class Node
  attr_accessor :next
end

def ring
  n1 = Node.new
  n2 = Node.new
  n3 = Node.new

  n1.next = n2
  n2.next = n3
  n3.next = n1

  n1
end

head = ring
```

This code is quite difficult to write with `std::rc::Rc` and `std::rc::Weak`
because the ring wants to own references. If we used `std::rc::Weak` to
implement `next`, after `ring` returns, `n2` and `n3` would be dropped and the
`std::rc::Weak`s in the object graph would be dangling.

`n1`, `n2`, and `n3` form a cycle. This cycle is _reachable_ because `n1` is
also bound to the variable `head`. The strong count of `n1` is two, which is
greater than the number of times it is owned by nodes in the cycle (only `n3`
owns `n1`). `n2` and `n3` should not be deallocated because they are in a cycle
with `n1`. Because `n1` is externally reachable, the entire cycle is externally
reachable.

If we instead write this code:

```ruby
head = ring
head = nil
# the cycle is unreachable and should be deallocated
```

The cycle is _orphaned_ because the only strong references to nodes in the cycle
come from other nodes in the cycle. The cycle is safe to deallocate and should
be reaped.

### Rust Example: Doubly Linked List

CactusRef can be used to
[implement a doubly linked list](https://github.com/lopopolo/ferrocarril/blob/53b4048628cd5577e378ce4fdae73a923340dcd1/cactusref/tests/no_leak_doubly_linked_list.rs)
with ergonomic strong references. The list is deallocated when the `list`
binding is dropped because the linked list is no longer externally reachable.

```rust
use cactusref::{Adoptable, Rc};
use std::cell::RefCell;
use std::iter;

struct Node<T> {
    pub prev: Option<Rc<RefCell<Self>>>,
    pub next: Option<Rc<RefCell<Self>>>,
    pub data: T,
}

struct List<T> {
    pub head: Option<Rc<RefCell<Node<T>>>>,
}

impl<T> List<T> {
    fn pop(&mut self) -> Option<Rc<RefCell<Node<T>>>> {
        let head = self.head.take()?;
        let tail = head.borrow_mut().prev.take();
        let next = head.borrow_mut().next.take();
        if let Some(ref tail) = tail {
            Rc::unadopt(&head, &tail);
            Rc::unadopt(&tail, &head);
            tail.borrow_mut().next = next.as_ref().map(Rc::clone);
            if let Some(ref next) = next {
                Rc::adopt(tail, next);
            }
        }
        if let Some(ref next) = next {
            Rc::unadopt(&head, &next);
            Rc::unadopt(&next, &head);
            next.borrow_mut().prev = tail.as_ref().map(Rc::clone);
            if let Some(ref tail) = tail {
                Rc::adopt(next, tail);
            }
        }
        self.head = next;
        Some(head)
    }
}

impl<T> From<Vec<T>> for List<T> {
    fn from(list: Vec<T>) -> Self {
        let nodes = list
            .into_iter()
            .map(|data| {
                Rc::new(RefCell::new(Node {
                    prev: None,
                    next: None,
                    data,
                }))
            })
            .collect::<Vec<_>>();
        for i in 0..nodes.len() - 1 {
            let curr = &nodes[i];
            let next = &nodes[i + 1];
            curr.borrow_mut().next = Some(Rc::clone(next));
            next.borrow_mut().prev = Some(Rc::clone(curr));
            Rc::adopt(curr, next);
            Rc::adopt(next, curr);
        }
        let tail = &nodes[nodes.len() - 1];
        let head = &nodes[0];
        tail.borrow_mut().next = Some(Rc::clone(head));
        head.borrow_mut().prev = Some(Rc::clone(tail));
        Rc::adopt(tail, head);
        Rc::adopt(head, tail);

        let head = Rc::clone(head);
        Self { head: Some(head) }
    }
}

let list = iter::repeat(())
    .map(|_| "a".repeat(1024 * 1024))
    .take(10)
    .collect::<Vec<_>>();
let mut list = List::from(list);
let head = list.pop().unwrap();
assert_eq!(Rc::strong_count(&head), 1);
assert_eq!(list.head.as_ref().map(Rc::strong_count), Some(3));
let weak = Rc::downgrade(&head);
drop(head);
assert!(weak.upgrade().is_none());
drop(list);
// all memory consumed by the list nodes is reclaimed.
```

### CactusRef Implementation

There are two magic pieces to CactusRef: `Rc` adoption and the cycle-busting
[`Drop`](https://lopopolo.github.io/ferrocarril/cactusref/struct.Rc.html#impl-Drop)
implementation.

#### Adoption

When an `Rc<T>` takes and holds an owned reference to another `Rc<T>`, calling
[`Rc::adopt`](https://lopopolo.github.io/ferrocarril/cactusref/struct.Rc.html#impl-Adoptable)
performs bookkeeping to build a graph of reachable objects. There is an
unlinking API, `Rc::unadopt`, which removes a reference from the graph.

An `Rc<T>` is able to adopt another `Rc<T>` multiple times. An `Rc<T>` is able
to adopt _itself_ multiple times. Together, these behaviors allow implementing
the following Ruby structure:

```ruby
ary = []
# => []
hash = { ary => ary }
# => {[]=>[]}
hash[hash] = hash
# => {[]=>[], {...}=>{...}}
ary << hash << hash << ary << ary
# => [{[...]=>[...], {...}=>{...}}, {[...]=>[...], {...}=>{...}}, [...], [...]]
hash = nil
ary = nil
# all structures are deallocated
```

This bookkeeping is implemented as a set of forward (owned) and backward (owned
by) links stored on the data structure that backs the `Rc` (called an
[`RcBox`](https://github.com/lopopolo/ferrocarril/blob/53b4048628cd5577e378ce4fdae73a923340dcd1/cactusref/src/ptr.rs#L84-L91)).

#### Drop

There are three states that `Rc` needs to deal with on `Drop` in this order:

1. `Rc` is unreachable and does not own any others. In this case,
   `Rc::strong_count` is zero and the set of forward links is empty.
2. `Rc` is part of an orphaned cycle. In this case, `Rc::strong_count` is
   greater than zero and the `Rc` has some forward or back links.
3. `Rc` is unreachable and has adopted links. In this case, `Rc::strong_count`
   is zero and the set of forward links is non-empty.

Each case is implemented with these steps:

1. Bust forward and back links on this `Rc`'s back links.
2. Bust forward and back links on this `Rc`.
3. Mark all reachable `Rc`s as killed.
4. Drop strong references.
5. Decrement the implicit "strong weak" pointer.
6. Deallocate.

The interesting case is state 2 which requires knowing whether this `Rc` is part
of an _orphaned cycle_. `Drop` detects whether this `Rc` is a member of a cycle
by performing breadth first search over the total set of forward and back links
in the object graph. The cycle detection algorithm tracks the reachability of
each node in the cycle by other cycle members. Forward links contribute toward
reachability. Backward references do not contribute but are added to the set of
nodes to traverse in the reachability analysis. Cycle detection is `O(links)`
where links is the number of active adoptions.

To determine whether the cycle is orphaned, the intra-cycle ownership counts are
compared to the strong count of each node. If the strong count for a node is
greater than the number of links the cycle has to that node, the node is
externally reachable and the cycle is not orphaned. Detecting an orphaned cycle
is `O(links + nodes)` where links is the number of active adoptions and nodes is
the number of `Rc`s in the cycle.

[Deallocating an orphaned cycle](https://github.com/lopopolo/ferrocarril/blob/53b4048628cd5577e378ce4fdae73a923340dcd1/cactusref/src/cycle/drop.rs#L163-L217)
is _fun_ and filled with unsafe peril. It is guaranteed that at least one other
object in the cycle owns a reference to this `Rc`, so as we deallocate members
of the cycle, this `Rc` will be dropped again.

Dropping this `Rc` multiple times is good because it manages decrementing the
strong count of this `Rc` automatically. This ensures that any outstanding
`Weak` pointers detect that they are dangling and return `None` on
`Weak::upgrade`. However, it will also certainly result in a
[double-free or use-after-free](https://en.wikipedia.org/wiki/C_dynamic_memory_allocation#Common_errors)
if we are not careful.

To avoid a double-free, the `RcBox` includes a `usize` field called `tombstone`.
When we attempt to drop an `Rc` in the cycle we
[mark it as killed](https://github.com/lopopolo/ferrocarril/blob/53b4048628cd5577e378ce4fdae73a923340dcd1/cactusref/src/cycle/drop.rs#L182-L193).
Subsequent calls to `drop` on killed `Rc`s early return after decrementing the
strong count.

To avoid a use-after-free, on drop, an `Rc`
[removes itself from all link tables](https://github.com/lopopolo/ferrocarril/blob/53b4048628cd5577e378ce4fdae73a923340dcd1/cactusref/src/cycle/drop.rs#L168-L181)
so it is not used for cycle detection.

To do the deallocation,
[drop the _values_ in the `Rc`s](https://github.com/lopopolo/ferrocarril/blob/53b4048628cd5577e378ce4fdae73a923340dcd1/cactusref/src/cycle/drop.rs#L194-L205)
instead of the `Rc`s. This breaks the cycle during the deallocation and allows
`Drop` to crawl the object graph.

### Cycle Detection Is a Zero-Cost Abstraction

Cycle detection is a zero-cost abstraction. If you never
`use cactusref::Adoptable;`, `Drop` uses the same implementation as
`std::rc::Rc` (and leaks in the same way as `std::rc::Rc` if you form a cycle of
strong references). The only costs you pay are the memory costs of one
[`Cell<usize>`](https://doc.rust-lang.org/nightly/core/cell/struct.Cell.html)
for preventing double frees, two empty
[`RefCell`](https://doc.rust-lang.org/nightly/core/cell/struct.RefCell.html)`<`[`HashMap`](https://doc.rust-lang.org/nightly/std/collections/struct.HashMap.html)`<NonNull<T>, usize>>`
for tracking adoptions, and an if statement to check if these structures are
empty on `drop`.

### Next Steps

I am [implementing a Ruby](https://github.com/lopopolo/ferrocarril) 💎 in Rust
and CactusRef will be used to implement the heap. CactusRef allows Ruby objects
to own strong references to their subordinate members (like instance variables,
keys and values in the case of a `Hash`, items in the case of an `Array`, class,
ancestor chain, and bound methods) and be automatically reaped once they become
unreachable in the VM.

CactusRef allows implementing a Ruby without a garbage collector, although if
you squint, CactusRef implements a tracing garbage collector using Rust's
built-in memory management.

Thank you [Stephen](https://github.com/tummychow) and
[Nelson](https://github.com/nelhage) for helping me think hard about algorithms.
😄

Thank you to the segfaults along the way for helping me find bugs in the cycle
detection and drop implementations. 😱

[^std-rc-api-compat]:

  CactusRef implements all `std::rc::Rc` APIs except for
  [`std::rc::Rc::downcast`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.downcast),
  [`CoerceUnsized`](https://doc.rust-lang.org/nightly/core/ops/trait.CoerceUnsized.html),
  and
  [`DispatchFromDyn`](https://doc.rust-lang.org/nightly/core/ops/trait.DispatchFromDyn.html).
