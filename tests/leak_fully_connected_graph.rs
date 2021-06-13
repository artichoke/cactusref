#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]

use cactusref::{Adopt, Rc};
use core::cell::RefCell;

struct Node<T> {
    _data: T,
    links: Vec<Rc<RefCell<Self>>>,
}

fn fully_connected_graph(count: usize) -> Vec<Rc<RefCell<Node<String>>>> {
    let mut nodes = vec![];
    for _ in 0..count {
        nodes.push(Rc::new(RefCell::new(Node {
            _data: "a".repeat(1024 * 1024),
            links: vec![],
        })));
    }
    for left in &nodes {
        for right in &nodes {
            let link = Rc::clone(right);
            unsafe {
                Rc::adopt(left, &link);
            }
            left.borrow_mut().links.push(link);
        }
    }
    nodes
}

#[test]
fn leak_fully_connected_graph() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("fully-connected graph");

    let list = fully_connected_graph(10);
    drop(Rc::clone(&list[0]));
    assert_eq!(Rc::strong_count(&list[0]), 11);
    drop(list);
}
