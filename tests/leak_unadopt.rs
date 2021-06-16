#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::cell::RefCell;

use cactusref::{Adopt, Rc};

struct S {
    inner: Option<Rc<RefCell<S>>>,
}

fn main() {
    env_logger::Builder::from_env("CACTUS_LOG").init();
}

#[test]
fn leak_unadopt() {
    log::info!("unadopt");

    let mut first = S { inner: None };
    let second = S { inner: None };
    let second = Rc::new(RefCell::new(second));

    first.inner = Some(Rc::clone(&second));
    let first = Rc::new(RefCell::new(first));
    unsafe {
        Rc::adopt_unchecked(&first, &second);
    }

    let inner = first.borrow_mut().inner.take().unwrap();
    for _ in 0..10 {
        Rc::unadopt(&first, &inner);
    }

    std::dbg!();
    drop(inner);
    std::dbg!();
    drop(first);
    std::dbg!();
}

#[test]
fn leak_with_elided_unadopt() {
    log::info!("unadopt");

    let mut first = S { inner: None };
    let second = S { inner: None };
    let second = Rc::new(RefCell::new(second));

    first.inner = Some(Rc::clone(&second));
    let first = Rc::new(RefCell::new(first));
    unsafe {
        Rc::adopt_unchecked(&first, &second);
    }

    let inner = first.borrow_mut().inner.take().unwrap();

    drop(inner);
    drop(first);
}
