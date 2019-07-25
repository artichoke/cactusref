#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings, intra_doc_link_resolution_failure)]
#![allow(clippy::shadow_unrelated)]

use cactusref::{Adoptable, Rc};
use std::cell::RefCell;

mod leak;

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
            Rc::adopt(left, &link);
            left.borrow_mut().links.push(link);
        }
    }
    nodes
}

#[test]
fn leak_fully_connected_graph() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    leak::Detector::new("fully-connected graph", None, None).check_leaks(|_| {
        let list = fully_connected_graph(10);
        drop(Rc::clone(&list[0]));
        assert_eq!(Rc::strong_count(&list[0]), 11);
        drop(list);
    });
}
