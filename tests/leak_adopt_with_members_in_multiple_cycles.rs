#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::cell::RefCell;

use cactusref::{Adopt, Rc};

struct NodeCell(RefCell<Node>);

impl NodeCell {
    fn new(data: String) -> Self {
        Self(RefCell::new(Node::new(data)))
    }

    fn link_to(&self, next: Rc<NodeCell>) {
        self.0.borrow_mut().link_to(next);
    }

    fn bridge_to(&self, other: Rc<NodeCell>) {
        self.0.borrow_mut().bridge_to(other);
    }
}

struct Node {
    data: String,
    next: Option<Rc<NodeCell>>,
    bridge: Option<Rc<NodeCell>>,
}

impl Node {
    fn new(data: String) -> Self {
        Self {
            data,
            next: None,
            bridge: None,
        }
    }

    fn link_to(&mut self, next: Rc<NodeCell>) {
        self.next = Some(next);
    }

    fn bridge_to(&mut self, other: Rc<NodeCell>) {
        self.bridge = Some(other);
    }
}

#[test]
fn leak_adopt_with_members_in_multiple_cycles() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("adopt with members in multiple cycles");

    let s = "a".repeat(1024 * 1024);

    let first = Rc::new(NodeCell::new(s.clone()));
    let link = Rc::clone(&first);
    unsafe {
        Rc::adopt_unchecked(&first, &link);
    }
    first.link_to(link);

    let group1 = first;

    let first = Rc::new(NodeCell::new(s));
    let link = Rc::clone(&first);
    unsafe {
        Rc::adopt_unchecked(&first, &link);
    }
    first.link_to(link);

    let group2 = first;
    // join the two cycles
    unsafe {
        Rc::adopt_unchecked(&group2, &group1);
        Rc::adopt_unchecked(&group1, &group2);
    }
    group2.bridge_to(Rc::clone(&group1));
    group1.bridge_to(Rc::clone(&group2));

    drop(group2);
    drop(group1);
}
