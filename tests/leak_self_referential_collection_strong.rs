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
fn leak_self_referential_collection_strong() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    let s = "a".repeat(2 * 1024 * 1024);

    leak::Detector::new("self-referential collection strong", None, None).check_leaks(|_| {
        let vec = Rc::new(RefCell::new(RArray {
            inner: vec![],
            _alloc: s.clone(),
        }));
        for _ in 1..10 {
            vec.borrow_mut().inner.push(Rc::clone(&vec));
            Rc::adopt(&vec, &vec);
        }
        drop(vec);
    });
}
