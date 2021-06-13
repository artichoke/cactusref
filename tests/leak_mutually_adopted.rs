#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use cactusref::{Adopt, Rc};

#[test]
fn leak_mutually_adopted() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    log::info!("mutually adopted");

    let s = "a".repeat(1024 * 1024);

    let first = Rc::new(s.clone());
    let last = Rc::new(s);
    unsafe {
        Rc::adopt(&first, &last);
        Rc::adopt(&last, &first);
    }
    drop(first);
    drop(last);
}
