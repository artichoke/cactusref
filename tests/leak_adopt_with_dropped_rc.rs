#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};

#[test]
fn leak_adopt_with_dropped_rc() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("adopt with dropped Rc");

    let s = "a".repeat(1024 * 1024);

    let first = Rc::new(s.clone());
    let mut last = Rc::clone(&first);
    for _ in 1..10 {
        let obj = Rc::new(s.clone());
        unsafe {
            Rc::adopt_unchecked(&obj, &last);
        }
        last = obj;
    }
    unsafe {
        Rc::adopt_unchecked(&first, &last);
    }
    drop(first);
    drop(last);
}
