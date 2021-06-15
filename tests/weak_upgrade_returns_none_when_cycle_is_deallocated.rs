#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};
use core::cell::RefCell;

struct RArray {
    inner: Vec<Rc<RefCell<Self>>>,
    alloc: String,
}

#[test]
fn weak_upgrade_returns_none_when_cycle_is_deallocated() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("Weak::upgrade on cycle drop");

    let s = "a".repeat(2 * 1024 * 1024);

    let vec = Rc::new(RefCell::new(RArray {
        inner: vec![],
        alloc: s,
    }));
    for _ in 0..10 {
        let clone = Rc::clone(&vec);
        unsafe {
            Rc::adopt_unchecked(&vec, &clone);
        }
        vec.borrow_mut().inner.push(clone);
    }
    assert_eq!(Rc::strong_count(&vec), 11);
    let weak = Rc::downgrade(&vec);
    assert!(weak.upgrade().is_some());
    assert!(weak.upgrade().unwrap().borrow().alloc.starts_with('a'));
    assert_eq!(weak.weak_count(), 1);
    drop(vec);
    assert!(weak.upgrade().is_none());
}
