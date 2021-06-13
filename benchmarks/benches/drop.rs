use std::cell::RefCell;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

use cactusref::{Adopt, Rc};

struct Node {
    links: Vec<Rc<RefCell<Self>>>,
}

fn chain_no_adoptions(count: usize) -> Rc<RefCell<Node>> {
    let first = Rc::new(RefCell::new(Node { links: vec![] }));
    let mut last = Rc::clone(&first);
    for _ in 1..count {
        let obj = Rc::new(RefCell::new(Node {
            links: vec![Rc::clone(&last)],
        }));
        last = obj;
    }
    last
}

fn chain_with_adoptions(count: usize) -> Rc<RefCell<Node>> {
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
    last
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

fn bench_drop_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("drop single");
    group.bench_function("zero-sized-type", |b| {
        b.iter_batched(|| Rc::new(()), drop, BatchSize::SmallInput)
    });
    group.bench_function("byte", |b| {
        b.iter_batched(|| Rc::new(0_u8), drop, BatchSize::SmallInput)
    });
    group.bench_function("u64", |b| {
        b.iter_batched(|| Rc::new(0_u64), drop, BatchSize::SmallInput)
    });
    group.bench_function("String", |b| {
        b.iter_batched(
            || Rc::new(String::from("bench")),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_chain_with_no_adoptions(c: &mut Criterion) {
    let mut group = c.benchmark_group("drop a chain with no adoptions");
    group.bench_function("10 nodes", |b| {
        b.iter_batched(
            || chain_no_adoptions(black_box(10)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("20 nodes", |b| {
        b.iter_batched(
            || chain_no_adoptions(black_box(20)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("30 nodes", |b| {
        b.iter_batched(
            || chain_no_adoptions(black_box(30)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("40 nodes", |b| {
        b.iter_batched(
            || chain_no_adoptions(black_box(40)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("50 nodes", |b| {
        b.iter_batched(
            || chain_no_adoptions(black_box(50)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("100 nodes", |b| {
        b.iter_batched(
            || chain_no_adoptions(black_box(100)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_chain_with_adoptions(c: &mut Criterion) {
    let mut group = c.benchmark_group("drop a chain with no adoptions");
    group.bench_function("10 nodes", |b| {
        b.iter_batched(
            || chain_with_adoptions(black_box(10)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("20 nodes", |b| {
        b.iter_batched(
            || chain_with_adoptions(black_box(20)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("30 nodes", |b| {
        b.iter_batched(
            || chain_with_adoptions(black_box(30)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("40 nodes", |b| {
        b.iter_batched(
            || chain_with_adoptions(black_box(40)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("50 nodes", |b| {
        b.iter_batched(
            || chain_with_adoptions(black_box(50)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("100 nodes", |b| {
        b.iter_batched(
            || chain_with_adoptions(black_box(100)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_circular_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("drop a circular graph");
    group.bench_function("10 nodes", |b| {
        b.iter_batched(
            || circular_graph(black_box(10)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("20 nodes", |b| {
        b.iter_batched(
            || circular_graph(black_box(20)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("30 nodes", |b| {
        b.iter_batched(
            || circular_graph(black_box(30)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("40 nodes", |b| {
        b.iter_batched(
            || circular_graph(black_box(40)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("50 nodes", |b| {
        b.iter_batched(
            || circular_graph(black_box(50)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("100 nodes", |b| {
        b.iter_batched(
            || circular_graph(black_box(100)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

fn bench_fully_connected_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("drop a fully connected graph");
    group.bench_function("10 nodes", |b| {
        b.iter_batched(
            || fully_connected_graph(black_box(10)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("20 nodes", |b| {
        b.iter_batched(
            || fully_connected_graph(black_box(20)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("30 nodes", |b| {
        b.iter_batched(
            || fully_connected_graph(black_box(30)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("40 nodes", |b| {
        b.iter_batched(
            || fully_connected_graph(black_box(40)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("50 nodes", |b| {
        b.iter_batched(
            || fully_connected_graph(black_box(50)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.bench_function("100 nodes", |b| {
        b.iter_batched(
            || fully_connected_graph(black_box(100)),
            drop,
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_drop_single,
    bench_chain_with_no_adoptions,
    bench_chain_with_adoptions,
    bench_circular_graph,
    bench_fully_connected_graph
);
criterion_main!(benches);
