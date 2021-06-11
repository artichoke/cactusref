use std::cell::RefCell;
use std::mem::drop;

use crate::{Rc, Weak};

#[test]
fn test_clone() {
    let x = Rc::new(RefCell::new(5));
    let y = x.clone();
    *x.borrow_mut() = 20;
    assert_eq!(*y.borrow(), 20);
}

#[test]
fn test_simple() {
    let x = Rc::new(5);
    assert_eq!(*x, 5);
}

#[test]
fn test_simple_clone() {
    let x = Rc::new(5);
    let y = x.clone();
    assert_eq!(*x, 5);
    assert_eq!(*y, 5);
}

#[test]
fn test_destructor() {
    let x: Rc<Box<_>> = Rc::new(Box::new(5));
    assert_eq!(**x, 5);
}

#[test]
fn test_live() {
    let x = Rc::new(5);
    let y = Rc::downgrade(&x);
    assert!(y.upgrade().is_some());
}

#[test]
fn test_dead() {
    let x = Rc::new(5);
    let y = Rc::downgrade(&x);
    drop(x);
    assert!(y.upgrade().is_none());
}

#[test]
fn is_unique() {
    let x = Rc::new(3);
    assert!(Rc::is_unique(&x));
    let y = x.clone();
    assert!(!Rc::is_unique(&x));
    drop(y);
    assert!(Rc::is_unique(&x));
    let w = Rc::downgrade(&x);
    assert!(!Rc::is_unique(&x));
    drop(w);
    assert!(Rc::is_unique(&x));
}

#[test]
fn test_strong_count() {
    let a = Rc::new(0);
    assert!(Rc::strong_count(&a) == 1);
    let w = Rc::downgrade(&a);
    assert!(Rc::strong_count(&a) == 1);
    let b = w.upgrade().expect("upgrade of live rc failed");
    assert!(Rc::strong_count(&b) == 2);
    assert!(Rc::strong_count(&a) == 2);
    drop(w);
    drop(a);
    assert!(Rc::strong_count(&b) == 1);
    let c = b.clone();
    assert!(Rc::strong_count(&b) == 2);
    assert!(Rc::strong_count(&c) == 2);
}

#[test]
fn test_weak_count() {
    let a = Rc::new(0);
    assert!(Rc::strong_count(&a) == 1);
    assert!(Rc::weak_count(&a) == 0);
    let w = Rc::downgrade(&a);
    assert!(Rc::strong_count(&a) == 1);
    assert!(Rc::weak_count(&a) == 1);
    drop(w);
    assert!(Rc::strong_count(&a) == 1);
    assert!(Rc::weak_count(&a) == 0);
    let c = a.clone();
    assert!(Rc::strong_count(&a) == 2);
    assert!(Rc::weak_count(&a) == 0);
    drop(c);
}

#[test]
fn weak_counts() {
    assert_eq!(Weak::weak_count(&Weak::<u64>::new()), 0);
    assert_eq!(Weak::strong_count(&Weak::<u64>::new()), 0);

    let a = Rc::new(0);
    let w = Rc::downgrade(&a);
    assert_eq!(Weak::strong_count(&w), 1);
    assert_eq!(Weak::weak_count(&w), 1);
    let w2 = w.clone();
    assert_eq!(Weak::strong_count(&w), 1);
    assert_eq!(Weak::weak_count(&w), 2);
    assert_eq!(Weak::strong_count(&w2), 1);
    assert_eq!(Weak::weak_count(&w2), 2);
    drop(w);
    assert_eq!(Weak::strong_count(&w2), 1);
    assert_eq!(Weak::weak_count(&w2), 1);
    let a2 = a.clone();
    assert_eq!(Weak::strong_count(&w2), 2);
    assert_eq!(Weak::weak_count(&w2), 1);
    drop(a2);
    drop(a);
    assert_eq!(Weak::strong_count(&w2), 0);
    assert_eq!(Weak::weak_count(&w2), 0);
    drop(w2);
}

#[test]
fn try_unwrap() {
    let x = Rc::new(3);
    assert_eq!(Rc::try_unwrap(x), Ok(3));
    let x = Rc::new(4);
    let _y = x.clone();
    assert_eq!(Rc::try_unwrap(x), Err(Rc::new(4)));
    let x = Rc::new(5);
    let _w = Rc::downgrade(&x);
    assert_eq!(Rc::try_unwrap(x), Ok(5));
}

#[test]
fn into_from_raw() {
    let x = Rc::new(Box::new("hello"));
    let y = x.clone();

    let x_ptr = Rc::into_raw(x);
    drop(y);
    unsafe {
        assert_eq!(**x_ptr, "hello");

        let x = Rc::from_raw(x_ptr);
        assert_eq!(**x, "hello");

        assert_eq!(Rc::try_unwrap(x).map(|x| *x), Ok("hello"));
    }
}

#[test]
fn into_from_weak_raw() {
    let x = Rc::new(Box::new("hello"));
    let y = Rc::downgrade(&x);

    let y_ptr = Weak::into_raw(y);
    unsafe {
        assert_eq!(**y_ptr, "hello");

        let y = Weak::from_raw(y_ptr);
        let y_up = Weak::upgrade(&y).unwrap();
        assert_eq!(**y_up, "hello");
        drop(y_up);

        assert_eq!(Rc::try_unwrap(x).map(|x| *x), Ok("hello"));
    }
}

#[test]
fn get_mut() {
    let mut x = Rc::new(3);
    *Rc::get_mut(&mut x).unwrap() = 4;
    assert_eq!(*x, 4);
    let y = x.clone();
    assert!(Rc::get_mut(&mut x).is_none());
    drop(y);
    assert!(Rc::get_mut(&mut x).is_some());
    let _w = Rc::downgrade(&x);
    assert!(Rc::get_mut(&mut x).is_none());
}

#[test]
fn test_cowrc_clone_make_unique() {
    let mut cow0 = Rc::new(75);
    let mut cow1 = cow0.clone();
    let mut cow2 = cow1.clone();

    assert!(75 == *Rc::make_mut(&mut cow0));
    assert!(75 == *Rc::make_mut(&mut cow1));
    assert!(75 == *Rc::make_mut(&mut cow2));

    *Rc::make_mut(&mut cow0) += 1;
    *Rc::make_mut(&mut cow1) += 2;
    *Rc::make_mut(&mut cow2) += 3;

    assert!(76 == *cow0);
    assert!(77 == *cow1);
    assert!(78 == *cow2);

    // none should point to the same backing memory
    assert!(*cow0 != *cow1);
    assert!(*cow0 != *cow2);
    assert!(*cow1 != *cow2);
}

#[test]
fn test_cowrc_clone_unique2() {
    let mut cow0 = Rc::new(75);
    let cow1 = cow0.clone();
    let cow2 = cow1.clone();

    assert!(75 == *cow0);
    assert!(75 == *cow1);
    assert!(75 == *cow2);

    *Rc::make_mut(&mut cow0) += 1;

    assert!(76 == *cow0);
    assert!(75 == *cow1);
    assert!(75 == *cow2);

    // cow1 and cow2 should share the same contents
    // cow0 should have a unique reference
    assert!(*cow0 != *cow1);
    assert!(*cow0 != *cow2);
    assert!(*cow1 == *cow2);
}

#[test]
fn test_cowrc_clone_weak() {
    let mut cow0 = Rc::new(75);
    let cow1_weak = Rc::downgrade(&cow0);

    assert!(75 == *cow0);
    assert!(75 == *cow1_weak.upgrade().unwrap());

    *Rc::make_mut(&mut cow0) += 1;

    assert!(76 == *cow0);
    assert!(cow1_weak.upgrade().is_none());
}

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
