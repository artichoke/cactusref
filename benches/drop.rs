#![deny(warnings, intra_doc_link_resolution_failure)]
#![deny(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate criterion;

use cactusref::{Adoptable, Rc};
use criterion::black_box;
use criterion::Criterion;
use std::cell::RefCell;

struct Node {
    links: Vec<Rc<RefCell<Self>>>,
}

fn circular_graph(count: usize) -> Rc<RefCell<Node>> {
    let first = Rc::new(RefCell::new(Node { links: vec![] }));
    let mut last = Rc::clone(&first);
    for _ in 1..count {
        let obj = Rc::new(RefCell::new(Node {
            links: vec![Rc::clone(&last)],
        }));
        unsafe {
            Rc::adopt(&obj, &last);
        }
        last = obj;
    }
    first.borrow_mut().links.push(Rc::clone(&last));
    unsafe {
        Rc::adopt(&first, &last);
    }
    first
}

fn fully_connected_graph(count: usize) -> Rc<RefCell<Node>> {
    let mut nodes = vec![];
    for _ in 0..count {
        nodes.push(Rc::new(RefCell::new(Node { links: vec![] })));
    }
    for left in &nodes {
        for right in &nodes {
            let link = Rc::clone(right);
            left.borrow_mut().links.push(link);
            unsafe {
                Rc::adopt(left, &right);
            }
        }
    }
    nodes.remove(0)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("drop single", |b| {
        b.iter_with_large_setup(|| Rc::new(()), drop)
    });
    c.bench_function_over_inputs(
        "drop a circular graph",
        |b, &&size| b.iter_with_large_setup(|| circular_graph(black_box(size)), drop),
        &[10, 20, 30, 40, 50, 100],
    );
    c.bench_function_over_inputs(
        "drop a fully connected graph",
        |b, &&size| b.iter_with_large_setup(|| fully_connected_graph(black_box(size)), drop),
        &[10, 20, 30, 40, 50, 100],
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
