use super::{Rc, Weak};
use std::boxed::Box;
use std::cell::RefCell;
use std::clone::Clone;
use std::convert::From;
use std::mem::drop;
use std::option::Option::{self, None, Some};
use std::result::Result::{Err, Ok};

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
fn weak_self_cyclic() {
    struct Cycle {
        x: RefCell<Option<Weak<Cycle>>>,
    }

    let a = Rc::new(Cycle {
        x: RefCell::new(None),
    });
    let b = Rc::downgrade(&a.clone());
    *a.x.borrow_mut() = Some(b);

    // hopefully we don't double-free (or leak)...
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
    assert_eq!(Weak::weak_count(&Weak::<u64>::new()), None);
    assert_eq!(Weak::strong_count(&Weak::<u64>::new()), 0);

    let a = Rc::new(0);
    let w = Rc::downgrade(&a);
    assert_eq!(Weak::strong_count(&w), 1);
    assert_eq!(Weak::weak_count(&w), Some(1));
    let w2 = w.clone();
    assert_eq!(Weak::strong_count(&w), 1);
    assert_eq!(Weak::weak_count(&w), Some(2));
    assert_eq!(Weak::strong_count(&w2), 1);
    assert_eq!(Weak::weak_count(&w2), Some(2));
    drop(w);
    assert_eq!(Weak::strong_count(&w2), 1);
    assert_eq!(Weak::weak_count(&w2), Some(1));
    let a2 = a.clone();
    assert_eq!(Weak::strong_count(&w2), 2);
    assert_eq!(Weak::weak_count(&w2), Some(1));
    drop(a2);
    drop(a);
    assert_eq!(Weak::strong_count(&w2), 0);
    assert_eq!(Weak::weak_count(&w2), Some(1));
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

// #[test]
// fn test_into_from_raw_unsized() {
//     let rc: Rc<str> = Rc::from("foo");
//
//     let ptr = Rc::into_raw(rc.clone());
//     let rc2 = unsafe { Rc::from_raw(ptr) };
//
//     assert_eq!(unsafe { &*ptr }, "foo");
//     assert_eq!(rc, rc2);
// }

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
    let item = Rc::new(75);
    assert_eq!(format!("{:?}", item), "75");
}

#[test]
fn test_from_owned() {
    let item = 123;
    let item_rc = Rc::from(item);
    assert!(123 == *item_rc);
}

#[test]
fn test_new_weak() {
    let item: Weak<usize> = Weak::new();
    assert!(item.upgrade().is_none());
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
fn test_from_str() {
    let r: Rc<str> = Rc::from("foo");

    assert_eq!(&r[..], "foo");
}

#[test]
fn test_copy_from_slice() {
    let s: &[u32] = &[1, 2, 3];
    let r: Rc<[u32]> = Rc::from(s);

    assert_eq!(&r[..], [1, 2, 3]);
}

#[test]
fn test_clone_from_slice() {
    #[derive(Clone, Debug, Eq, PartialEq)]
    struct X(u32);

    let s: &[X] = &[X(1), X(2), X(3)];
    let r: Rc<[X]> = Rc::from(s);

    assert_eq!(&r[..], s);
}

#[test]
#[should_panic]
fn test_clone_from_slice_panic() {
    use std::string::{String, ToString};

    struct Fail(u32, String);

    impl Clone for Fail {
        fn clone(&self) -> Self {
            if self.0 == 2 {
                panic!();
            }
            Self(self.0, self.1.clone())
        }
    }

    let s: &[Fail] = &[
        Fail(0, "foo".to_string()),
        Fail(1, "bar".to_string()),
        Fail(2, "baz".to_string()),
    ];

    // Should panic, but not cause memory corruption
    let _r: Rc<[Fail]> = Rc::from(s);
}

#[test]
fn test_from_box() {
    let b: Box<u32> = Box::new(123);
    let r: Rc<u32> = Rc::from(b);

    assert_eq!(*r, 123);
}

#[test]
fn test_from_box_str() {
    use std::string::String;

    let s = String::from("foo").into_boxed_str();
    let r: Rc<str> = Rc::from(s);

    assert_eq!(&r[..], "foo");
}

#[test]
fn test_from_box_slice() {
    let s = vec![1, 2, 3].into_boxed_slice();
    let r: Rc<[u32]> = Rc::from(s);

    assert_eq!(&r[..], [1, 2, 3]);
}

#[test]
fn test_from_box_trait() {
    use std::fmt::Display;
    use std::string::ToString;

    let b: Box<dyn Display> = Box::new(123);
    let r: Rc<dyn Display> = Rc::from(b);

    assert_eq!(r.to_string(), "123");
}

#[test]
fn test_from_box_trait_zero_sized() {
    use std::fmt::Debug;

    let b: Box<dyn Debug> = Box::new(());
    let r: Rc<dyn Debug> = Rc::from(b);

    assert_eq!(format!("{:?}", r), "()");
}

#[test]
fn test_from_vec() {
    let v = vec![1, 2, 3];
    let r: Rc<[u32]> = Rc::from(v);

    assert_eq!(&r[..], [1, 2, 3]);
}
