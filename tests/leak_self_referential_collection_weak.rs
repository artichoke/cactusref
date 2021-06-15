#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc, Weak};
use core::cell::RefCell;

struct RArray {
    inner: Vec<Weak<RefCell<Self>>>,
    alloc: String,
}

#[test]
fn leak_self_referential_collection_weak() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("self-referential collection weak");

    let s = "a".repeat(2 * 1024 * 1024);

    // each iteration creates 2MB of empty buffers
    let vec = Rc::new(RefCell::new(RArray {
        inner: vec![],
        alloc: s,
    }));
    for _ in 1..10 {
        vec.borrow_mut().inner.push(Rc::downgrade(&vec));
        unsafe {
            Rc::adopt_unchecked(&vec, &vec);
        }
    }
    let borrow = vec.borrow();
    let mut iter = borrow.inner.iter();
    let valid = iter.all(|elem| {
        let upgrade = elem.upgrade();
        let rc = upgrade.unwrap();
        let rarray = rc.borrow();
        let starts_with = rarray.alloc.starts_with('a');
        drop(rarray);
        assert_eq!(Rc::strong_count(&rc), 2);
        drop(rc);
        starts_with
    });
    assert!(valid);
    drop(borrow);
    drop(vec);
}
