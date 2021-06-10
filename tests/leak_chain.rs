#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::Rc;
use core::cell::RefCell;

struct RString {
    inner: String,
    link: Option<Rc<RefCell<Self>>>,
}

#[test]
fn leak_chain() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("chain");

    let s = "a".repeat(1024 * 1024);

    let first = Rc::new(RefCell::new(RString {
        inner: s.clone(),
        link: None,
    }));
    let mut last = Rc::clone(&first);
    for _ in 1..10 {
        let obj = Rc::new(RefCell::new(RString {
            inner: s.clone(),
            link: Some(Rc::clone(&last)),
        }));
        last = obj;
    }
    assert!(first.borrow().link.is_none());
    assert_eq!(first.borrow().inner, s);
    assert!(last.borrow().link.is_some());
    assert_eq!(last.borrow().inner, s);
    drop(first);
    drop(last);
}
