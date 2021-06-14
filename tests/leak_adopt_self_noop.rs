#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};
use core::cell::RefCell;

struct RString {
    inner: String,
    link: Option<Rc<RefCell<Self>>>,
}

#[test]
fn adopt_self_noop() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("adopt self");

    let s = "a".repeat(1024 * 1024 * 5);

    let first = Rc::new(RefCell::new(RString {
        inner: s.clone(),
        link: None,
    }));
    unsafe {
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
        Rc::adopt(&first, &first);
    }
    assert_eq!(first.borrow().inner, s);
    assert!(first.borrow().link.is_none());
    drop(first);
}
