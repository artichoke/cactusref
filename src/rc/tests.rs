use std::cell::RefCell;
use std::mem::drop;

use crate::{Rc, Weak};

#[test]
fn test_show() {
    let foo = Rc::new(75);
    assert_eq!(format!("{:?}", foo), "75");
}

#[test]
fn test_from_owned() {
    let foo = 123;
    let foo_rc = Rc::from(foo);
    assert!(123 == *foo_rc);
}

#[test]
fn test_new_weak() {
    let foo: Weak<usize> = Weak::new();
    assert!(foo.upgrade().is_none());
}

#[test]
fn test_ptr_eq() {
    let five = Rc::new(5);
    let same_five = five.clone();
    let other_five = Rc::new(5);

    assert!(Rc::ptr_eq(&five, &same_five));
    assert!(!Rc::ptr_eq(&five, &other_five));
}

#[test]
fn test_from_box() {
    let b: Box<u32> = Box::new(123);
    let r: Rc<u32> = Rc::from(b);

    assert_eq!(*r, 123);
}
