#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![forbid(unsafe_code)]

use cactusref::{Adopt, Rc, Trace};
use core::cell::RefCell;
use core::iter;
use core::ops::Deref;

struct NodeCell<T>(RefCell<Node<T>>);

impl<T> NodeCell<T> {
    fn new(data: T) -> Self {
        Self(RefCell::new(Node {
            prev: None,
            next: None,
            data,
        }))
    }
}

impl<T> Deref for NodeCell<T> {
    type Target = RefCell<Node<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct Node<T> {
    pub prev: Option<Rc<NodeCell<T>>>,
    pub next: Option<Rc<NodeCell<T>>>,
    pub data: T,
}

impl<T> Trace for NodeCell<T> {
    fn yield_owned_rcs<F>(&self, mut mark: F)
    where
        F: FnMut(&mut Rc<Self>),
    {
        if let Some(ref mut prev) = self.borrow_mut().prev {
            mark(prev);
        }
        if let Some(ref mut next) = self.borrow_mut().next {
            mark(next);
        }
    }
}

struct List<T> {
    pub head: Option<Rc<NodeCell<T>>>,
}

impl<T> List<T> {
    fn pop(&mut self) -> Option<Rc<NodeCell<T>>> {
        let head = self.head.take()?;
        let mut tail = head.borrow_mut().prev.take();
        let mut next = head.borrow_mut().next.take();

        if let Some(ref mut tail) = tail {
            Rc::unadopt(&head, tail);
            Rc::unadopt(tail, &head);

            tail.borrow_mut().next = next.as_ref().map(Rc::clone);
            if let Some(ref next) = next {
                Rc::adopt(tail, next);
            }
        }

        if let Some(ref mut next) = next {
            Rc::unadopt(&head, next);
            Rc::unadopt(next, &head);

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
        if list.is_empty() {
            return Self { head: None };
        }
        let mut nodes = list
            .into_iter()
            .map(|data| Rc::new(NodeCell::new(data)))
            .collect::<Vec<_>>();

        for i in 0..nodes.len() - 1 {
            let next = Rc::clone(&nodes[i + 1]);
            let curr = &mut nodes[i];
            curr.borrow_mut().next = Some(Rc::clone(&next));
            Rc::adopt(curr, &next);

            let curr = Rc::clone(&nodes[i]);
            let next = &mut nodes[i + 1];
            next.borrow_mut().prev = Some(Rc::clone(&curr));
            Rc::adopt(next, &curr);
        }

        let head = Rc::clone(&nodes[0]);
        let tail = nodes.last_mut().unwrap();
        tail.borrow_mut().next = Some(Rc::clone(&head));
        Rc::adopt(tail, &head);

        let tail = Rc::clone(&nodes[nodes.len() - 1]);
        let head = &mut nodes[0];
        head.borrow_mut().prev = Some(Rc::clone(&tail));
        Rc::adopt(head, &tail);

        let head = Rc::clone(head);
        Self { head: Some(head) }
    }
}

#[test]
fn leak_doubly_linked_list() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("doubly linked list");

    let list = iter::repeat(())
        .map(|_| "a".repeat(1024 * 1024))
        .take(10)
        .collect::<Vec<_>>();
    let mut list = List::from(list);

    let head = list.pop().unwrap();
    assert!(head.borrow().data.starts_with('a'));
    assert_eq!(Rc::strong_count(&head), 1);
    assert_eq!(list.head.as_ref().map(Rc::strong_count), Some(3));

    let weak = Rc::downgrade(&head);
    drop(head);
    assert!(weak.upgrade().is_none());
    drop(list);
}
