#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};
use core::cell::RefCell;

struct RString {
    inner: String,
    link: Option<Rc<RefCell<Self>>>,
}

#[test]
fn leak_adopt_self() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("adopt self");

    let s = "a".repeat(1024 * 1024 * 5);

    let first = Rc::new(RefCell::new(RString {
        inner: s.clone(),
        link: None,
    }));
    let clone = Rc::clone(&first);
    unsafe {
        Rc::adopt(&first, &clone);
        Rc::adopt(&first, &clone);
        Rc::adopt(&first, &clone);
        Rc::adopt(&first, &clone);
        Rc::adopt(&first, &clone);
        Rc::adopt(&first, &clone);
        Rc::adopt(&first, &clone);
        Rc::adopt(&first, &clone);
    }
    first.borrow_mut().link = Some(clone);
    assert_eq!(first.borrow().inner, s);
    assert!(first.borrow().link.is_some());
    drop(first);
}
