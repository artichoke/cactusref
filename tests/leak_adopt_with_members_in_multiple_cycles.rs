#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};

#[test]
fn leak_adopt_with_members_in_multiple_cycles() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("adopt with members in multiple cycles");

    let s = "a".repeat(1024 * 1024);

    let first = Rc::new(s.clone());
    let mut last = Rc::clone(&first);
    for _ in 1..10 {
        let obj = Rc::new(s.clone());
        unsafe {
            Rc::adopt(&obj, &last);
        }
        last = obj;
    }
    unsafe {
        Rc::adopt(&first, &last);
    }
    let group1 = first;
    let first = Rc::new(s.clone());
    let mut last = Rc::clone(&first);
    for _ in 101..110 {
        let obj = Rc::new(s.clone());
        unsafe {
            Rc::adopt(&obj, &last);
        }
        last = obj;
    }
    unsafe {
        Rc::adopt(&first, &last);
    }
    let group2 = first;
    // join the two cycles
    unsafe {
        Rc::adopt(&group2, &group1);
        Rc::adopt(&group1, &group2);
    }
    drop(last);
    drop(group2);
    drop(group1);
}
