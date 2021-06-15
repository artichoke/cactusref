#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};
use std::cell::RefCell;

#[derive(Default)]
struct Array {
    buffer: Vec<Rc<RefCell<Self>>>,
}

#[test]
fn weak() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    let array = Rc::new(RefCell::new(Array::default()));
    for _ in 0..10 {
        let item = Rc::clone(&array);
        unsafe {
            Rc::adopt_unchecked(&array, &item);
        }
        array.borrow_mut().buffer.push(item);
    }
    assert_eq!(Rc::strong_count(&array), 11);
    assert_eq!(Rc::weak_count(&array), 0);

    let weak = Rc::downgrade(&array);
    assert!(weak.upgrade().is_some());
    assert_eq!(weak.weak_count(), 1);
    assert_eq!(weak.upgrade().as_ref().map(Rc::strong_count), Some(12));
    assert_eq!(weak.strong_count(), 11);
    assert_eq!(weak.weak_count(), 1);
    assert_eq!(weak.upgrade().unwrap().borrow().buffer.len(), 10);

    // 1 for the array binding, 10 for the `Rc`s in buffer, and 10
    // for the self adoptions.
    assert_eq!(Rc::strong_count(&array), 11);

    drop(array);

    assert_eq!(weak.strong_count(), 0);
    assert_eq!(weak.weak_count(), 0);
    assert!(weak.upgrade().is_none());
}
