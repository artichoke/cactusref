#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adoptable, Rc, Weak};
use core::cell::RefCell;

struct RArray {
    inner: Vec<Weak<RefCell<Self>>>,
    _alloc: String,
}

#[test]
fn leak_self_referential_collection_weak() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("self-referential collection weak");

    let s = "a".repeat(2 * 1024 * 1024);

    // each iteration creates 2MB of empty buffers
    let vec = Rc::new(RefCell::new(RArray {
        inner: vec![],
        _alloc: s.clone(),
    }));
    for _ in 1..10 {
        vec.borrow_mut().inner.push(Rc::downgrade(&vec));
        unsafe {
            Rc::adopt(&vec, &vec);
        }
    }
    drop(vec);
}
