#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings, intra_doc_link_resolution_failure)]

use cactusref::{Adoptable, Rc};
use std::cell::RefCell;

mod leak;

struct RArray {
    inner: Vec<Rc<RefCell<Self>>>,
    _alloc: String,
}

#[test]
fn weak_upgrade_returns_none_when_cycle_is_deallocated() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    let s = "a".repeat(2 * 1024 * 1024);

    leak::Detector::new("Weak::upgrade on cycle drop", None, None).check_leaks(|_| {
        let vec = Rc::new(RefCell::new(RArray {
            inner: vec![],
            _alloc: s.clone(),
        }));
        for _ in 0..10 {
            vec.borrow_mut().inner.push(Rc::clone(&vec));
            Rc::adopt(&vec, &vec);
        }
        assert_eq!(Rc::strong_count(&vec), 11);
        let weak = Rc::downgrade(&vec);
        assert!(weak.upgrade().is_some());
        assert_eq!(weak.weak_count(), Some(1));
        drop(vec);
        assert!(weak.upgrade().is_none());
    });
}
