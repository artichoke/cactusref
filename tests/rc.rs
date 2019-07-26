use cactusref::{Rc, Weak};
use std::any::Any;
use std::cell::RefCell;
use std::cmp::PartialEq;

#[test]
fn uninhabited() {
    enum Void {}
    let mut a = Weak::<Void>::new();
    a = a.clone();
    assert!(a.upgrade().is_none());
}

#[test]
fn float_nan_ne() {
    let x = Rc::new(std::f32::NAN);
    assert!(x != x);
    assert!(!(x == x));
}

#[test]
fn partial_eq() {
    struct TestPEq(RefCell<usize>);
    impl PartialEq for TestPEq {
        fn eq(&self, other: &TestPEq) -> bool {
            *self.0.borrow_mut() += 1;
            *other.0.borrow_mut() += 1;
            true
        }
    }
    let x = Rc::new(TestPEq(RefCell::new(0)));
    assert!(x == x);
    assert!(!(x != x));
    assert_eq!(*x.0.borrow(), 4);
}

#[test]
fn eq() {
    #[derive(Eq)]
    struct TestEq(RefCell<usize>);
    impl PartialEq for TestEq {
        fn eq(&self, other: &TestEq) -> bool {
            *self.0.borrow_mut() += 1;
            *other.0.borrow_mut() += 1;
            true
        }
    }
    let x = Rc::new(TestEq(RefCell::new(0)));
    assert!(x == x);
    assert!(!(x != x));
    assert_eq!(*x.0.borrow(), 0);
}
