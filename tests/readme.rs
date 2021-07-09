use cactusref::{Adopt, Rc};
use std::cell::RefCell;

struct Node {
    next: Option<Rc<RefCell<Node>>>,
    data: i32,
}

#[test]
fn readme() {
    let left = Node {
        next: None,
        data: 123,
    };
    let left = Rc::new(RefCell::new(left));

    let right = Node {
        next: Some(Rc::clone(&left)),
        data: 456,
    };
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
        println!(
            "traversing ring and found node with data = {}",
            node.borrow().data
        );
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
}
