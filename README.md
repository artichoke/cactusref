# 🌵 `CactusRef`

Single-threaded, cycle-aware, reference-counting pointers. 'Rc' stands for
'Reference Counted'.

[![CircleCI](https://circleci.com/gh/artichoke/cactusref.svg?style=svg)](https://circleci.com/gh/artichoke/cactusref)
[![Documentation](https://img.shields.io/badge/docs-cactusref-blue.svg)](https://artichoke.github.io/cactusref/cactusref/)

The type
[`Rc<T>`](https://artichoke.github.io/cactusref/cactusref/struct.Rc.html)
provides shared ownership of a value of type `T`, allocated in the heap.
Invoking
[`clone`](https://artichoke.github.io/cactusref/cactusref/struct.Rc.html#impl-Clone)
on [`Rc`](https://artichoke.github.io/cactusref/cactusref/struct.Rc.html)
produces a new pointer to the same value in the heap. When the last externally
reachable [`Rc`](https://artichoke.github.io/cactusref/cactusref/struct.Rc.html)
pointer to a given value is destroyed, the pointed-to value is also destroyed.

`Rc` can **detect and deallocate cycles** of `Rc`s through the use of
[`Adoptable`](https://artichoke.github.io/cactusref/cactusref/trait.Adoptable.html).
Cycle detection is a zero-cost abstraction.

🌌 `CactusRef` depends on _several_ unstable Rust features and can only be built
on nightly. `CactusRef` implements
[`std::rc`](https://doc.rust-lang.org/std/rc/index.html)'s pinning APIs which
requires at least Rust 1.33.0.

## `CactusRef` vs. `std::rc`

The `Rc` in `CactusRef` is derived from
[`std::rc::Rc`](https://doc.rust-lang.org/std/rc/struct.Rc.html) and `CactusRef`
implements most of the API from `std`.

`cactusref::Rc` does not implement the following APIs that are present on
[`rc::Rc`](https://doc.rust-lang.org/std/rc/struct.Rc.html):

- [`std::rc::Rc::downcast`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.downcast)
- [`CoerceUnsized`](https://doc.rust-lang.org/nightly/core/ops/trait.CoerceUnsized.html)
- [`DispatchFromDyn`](https://doc.rust-lang.org/nightly/core/ops/trait.DispatchFromDyn.html)

If you do not depend on these APIs, `cactusref` is a drop-in replacement for
[`std::rc`](https://doc.rust-lang.org/std/rc/index.html).

Like [`std::rc`](https://doc.rust-lang.org/std/rc/index.html),
[`Rc`](https://artichoke.github.io/cactusref/cactusref/struct.Rc.html) and
[`Weak`](https://artichoke.github.io/cactusref/cactusref/struct.Weak.html) are
`!`[`Send`](https://doc.rust-lang.org/nightly/core/marker/trait.Send.html) and
`!`[`Sync`](https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html).

## Cycle Detection

`Rc` implements
[`Adoptable`](https://artichoke.github.io/cactusref/cactusref/trait.Adoptable.html)
to log bookkeeping entries for strong ownership links to other `Rc`s that may
form a cycle. The ownership links tracked by these bookkeeping entries form an
object graph of reachable `Rc`s. On `drop`, `Rc` uses these entries to conduct a
reachability trace of the object graph to determine if it is part of an
_orphaned cycle_. An orphaned cycle is a cycle where the only strong references
to all nodes in the cycle come from other nodes in the cycle.

Cycle detection is a zero-cost abstraction. If you never
`use cactusref::Adoptable;`, `Drop` uses the same implementation as
`std::rc::Rc` (and leaks in the same way as `std::rc::Rc` if you form a cycle of
strong references). The only costs you pay are the memory costs of one empty
[`RefCell`](https://doc.rust-lang.org/nightly/core/cell/struct.RefCell.html)`<`[`HashMap`](https://docs.rs/hashbrown/0.5.0/hashbrown/struct.HashMap.html)`<NonNull<T>, usize>>`
for tracking adoptions and an if statement to check if these structures are
empty on `drop`.

Cycle detection uses breadth-first search for traversing the object graph. The
algorithm supports arbitrarily large object graphs and will not overflow the
stack during the reachability trace.

## Self-Referential Structures

`CactusRef` can be used to implement collections that own strong references to
themselves. The following implements a doubly-linked list that is fully
deallocated once the `list` binding is dropped.

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
            unsafe {
                Rc::unadopt(&head, &tail);
                Rc::unadopt(&tail, &head);
            }
            tail.borrow_mut().next = next.as_ref().map(Rc::clone);
            if let Some(ref next) = next {
                unsafe {
                    Rc::adopt(tail, next);
                }
            }
        }
        if let Some(ref next) = next {
            unsafe {
                Rc::unadopt(&head, &next);
                Rc::unadopt(&next, &head);
            }
            next.borrow_mut().prev = tail.as_ref().map(Rc::clone);
            if let Some(ref tail) = tail {
                unsafe {
                    Rc::adopt(next, tail);
                }
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
            unsafe {
                Rc::adopt(curr, next);
                Rc::adopt(next, curr);
            }
        }
        let tail = &nodes[nodes.len() - 1];
        let head = &nodes[0];
        tail.borrow_mut().next = Some(Rc::clone(head));
        head.borrow_mut().prev = Some(Rc::clone(tail));
        unsafe {
            Rc::adopt(tail, head);
            Rc::adopt(head, tail);
        }

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

## License

CactusRef is licensed with the [MIT License](/LICENSE) (c) Ryan Lopopolo.

CactusRef is derived from `Rc` in the Rust standard library @
[`296e825`](https://github.com/rust-lang/rust/blob/296e825afab8665dfc5527aa8f72dfe5f5894224/src/liballoc/rc.rs)
which is dual licensed with the
[MIT License](https://github.com/rust-lang/rust/blob/master/LICENSE-MIT) and
[Apache 2.0 License](https://github.com/rust-lang/rust/blob/master/LICENSE-APACHE).
