#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};
use core::cell::RefCell;

struct RArray {
    inner: Vec<Rc<RefCell<Self>>>,
    _alloc: String,
}

#[test]
fn leak_self_referential_collection_strong() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("self-referential collection strong");

    let s = "a".repeat(2 * 1024 * 1024);

    let vec = Rc::new(RefCell::new(RArray {
        inner: vec![],
        _alloc: s.clone(),
    }));
    for _ in 1..10 {
        vec.borrow_mut().inner.push(Rc::clone(&vec));
        unsafe {
            Rc::adopt(&vec, &vec);
        }
    }
    drop(vec);
}
