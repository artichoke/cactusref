#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings, intra_doc_link_resolution_failure)]

use cactusref::{Adoptable, Rc};
use std::cell::RefCell;

mod leak;

struct RString {
    inner: String,
    link: Option<Rc<RefCell<Self>>>,
}

#[test]
fn leak_adopt_self() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    let s = "a".repeat(1024 * 1024 * 5);

    leak::Detector::new("adopt self", None, None).check_leaks(|_| {
        let first = Rc::new(RefCell::new(RString {
            inner: s.clone(),
            link: None,
        }));
        first.borrow_mut().link = Some(Rc::clone(&first));
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        assert_eq!(first.borrow().inner, s);
        drop(first);
    });
}
