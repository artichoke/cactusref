#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};
use core::cell::RefCell;

struct RArray {
    inner: Vec<Rc<RefCell<Self>>>,
    _alloc: String,
}

#[test]
fn weak_upgrade_returns_none_when_cycle_is_deallocated() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("Weak::upgrade on cycle drop");

    let s = "a".repeat(2 * 1024 * 1024);

    let vec = Rc::new(RefCell::new(RArray {
        inner: vec![],
        _alloc: s.clone(),
    }));
    for _ in 0..10 {
        vec.borrow_mut().inner.push(Rc::clone(&vec));
        unsafe {
            Rc::adopt(&vec, &vec);
        }
    }
    assert_eq!(Rc::strong_count(&vec), 11);
    let weak = Rc::downgrade(&vec);
    assert!(weak.upgrade().is_some());
    assert_eq!(weak.weak_count(), 1);
    drop(vec);
    assert!(weak.upgrade().is_none());
}
