#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};
use core::cell::RefCell;

struct RArray {
    inner: Vec<Rc<RefCell<Self>>>,
    alloc: String,
}

#[test]
fn leak_self_referential_collection_strong() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("self-referential collection strong");

    let s = "a".repeat(2 * 1024 * 1024);

    let vec = Rc::new(RefCell::new(RArray {
        inner: vec![],
        alloc: s,
    }));
    for _ in 1..10 {
        let clone = Rc::clone(&vec);
        unsafe {
            Rc::adopt(&vec, &clone);
        }
        vec.borrow_mut().inner.push(clone);
    }
    let borrow = vec.borrow();
    let mut iter = borrow.inner.iter();
    let valid = iter.all(|elem| {
        let rarray = elem.borrow();
        let starts_with = rarray.alloc.starts_with('a');
        drop(rarray);
        assert_eq!(Rc::strong_count(elem), 10);
        starts_with
    });
    assert!(valid);
    drop(borrow);
    drop(vec);
}
