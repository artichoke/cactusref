# CactusRef

[![GitHub Actions](https://github.com/artichoke/cactusref/workflows/CI/badge.svg)](https://github.com/artichoke/cactusref/actions)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/cactusref.svg)](https://crates.io/crates/cactusref)
[![API](https://docs.rs/cactusref/badge.svg)](https://docs.rs/cactusref)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/cactusref/cactusref/)

Single-threaded, cycle-aware, reference-counting pointers. 'Rc' stands for
'Reference Counted'.

> What if, hear me out, we put a hash map in a smart pointer?

CactusRef is a single-threaded, reference-counted smart pointer that can
deallocate cycles without having to resort to weak pointers. [`Rc`][std-rc] from
`std` can be difficult to work with because creating a cycle of `Rc`s will
result in a memory leak.

[std-rc]: https://doc.rust-lang.org/stable/std/rc/struct.Rc.html

CactusRef is a near drop-in replacement for `std::rc::Rc` which introduces
additional APIs for bookkeeping ownership relationships in a graph of `Rc`s.

Combining CactusRef's [adoption APIs] for tracking links in the object graph and
driving garbage collection with Rust's [drop glue] implements a kind of tracing
garbage collector. Graphs of CactusRefs detect cycles local to the graph of
connected CactusRefs and do not need to scan the whole heap as is [typically
required][rust-tour-tracing-gc] in a tracing garbage collector.

Cycles of CactusRefs are deterministically collected and deallocated when they
are no longer reachable from outside of the cycle.

[adoption apis]:
  https://artichoke.github.io/cactusref/cactusref/trait.Adopt.html
[drop glue]: https://doc.rust-lang.org/nightly/reference/destructors.html
[rust-tour-tracing-gc]:
  https://manishearth.github.io/blog/2021/04/05/a-tour-of-safe-tracing-gc-designs-in-rust/

## Self-referential Data Structures

CactusRef can be used to implement [self-referential data structures] such as a
doubly-linked list without using weak references.

[self-referential data structures]:
  https://artichoke.github.io/cactusref/cactusref/implementing_self_referential_data_structures/index.html

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cactusref = "0.3.0"
```

CactusRef is mostly a drop-in replacement for `std::rc::Rc`, which can be used
like:

```rust
use cactusref::Rc;

let node = Rc::new(123_i32);
let another = Rc::clone(&node);
assert_eq!(Rc::strong_count(&another), 2);

let weak = Rc::downgrade(&node);
assert!(weak.upgrade().is_some());
```

Or start making self-referential data structures like:

```rust
use std::cell::RefCell;
use cactusref::{Adopt, Rc};

struct Node {
    next: Option<Rc<RefCell<Node>>>,
    data: i32,
}

let left = Node { next: None, data: 123 };
let left = Rc::new(RefCell::new(left));

let right = Node { next: Some(Rc::clone(&left)), data: 456 };
let right = Rc::new(RefCell::new(right));

unsafe {
    // bookkeep that `right` has added an owning ref to `left`.
    Rc::adopt_unchecked(&right, &left);
}

left.borrow_mut().next = Some(Rc::clone(&right));

unsafe {
    // bookkeep that `left` has added an owning ref to `right`.
    Rc::adopt_unchecked(&left, &right);
}

let mut node = Rc::clone(&left);
// this loop will print:
//
// > traversing ring and found node with data = 123
// > traversing ring and found node with data = 456
// > traversing ring and found node with data = 123
// > traversing ring and found node with data = 456
// > traversing ring and found node with data = 123
for _ in 0..5 {
    println!("traversing ring and found node with data = {}", node.borrow().data);
    let next = if let Some(ref next) = node.borrow().next {
        Rc::clone(next)
    } else {
        break;
    };
    node = next;
}
assert_eq!(Rc::strong_count(&node), 3);
drop(node);

drop(left);
drop(right);
// All members of the ring are garbage collected and deallocated.
```

## Maturity

CactusRef is experimental. This crate has several limitations:

- CactusRef is nightly only.
- Cycle detection requires [unsafe code][adopt-api] to use.

CactusRef is a non-trivial extension to `std::rc::Rc` and has not been proven to
be safe. Although CactusRef makes a best effort to abort the program if it
detects a dangling `Rc`, this crate may be unsound.

[adopt-api]: https://docs.rs/cactusref/*/cactusref/trait.Adopt.html

## `no_std`

CactusRef is `no_std` compatible with an optional and enabled by default
dependency on `std`. CactusRef depends on the [`alloc`] crate.

[`alloc`]: https://doc.rust-lang.org/alloc/

## Crate features

All features are enabled by default.

- **std** - Enable linking to the [Rust Standard Library]. Enabling this feature
  adds [`Error`] implementations to error types in this crate.

[rust standard library]: https://doc.rust-lang.org/nightly/std/
[`error`]: https://doc.rust-lang.org/nightly/std/error/trait.Error.html

## License

CactusRef is licensed with the [MIT License](LICENSE) (c) Ryan Lopopolo.

CactusRef is derived from `Rc` in the Rust standard library @
[`f586d79d`][alloc-rc-snapshot] which is dual licensed with the [MIT
License][rust-mit-license] and [Apache 2.0 License][rust-apache2-license].

[alloc-rc-snapshot]:
  https://github.com/rust-lang/rust/blob/f586d79d183d144e0cbf519e29247f36670e2076/library/alloc/src/rc.rs
[rust-mit-license]:
  https://github.com/rust-lang/rust/blob/f586d79d183d144e0cbf519e29247f36670e2076/LICENSE-MIT
[rust-apache2-license]:
  https://github.com/rust-lang/rust/blob/f586d79d183d144e0cbf519e29247f36670e2076/LICENSE-APACHE
