#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings, intra_doc_link_resolution_failure)]

use cactusref::{Adoptable, Rc};

mod leak;

#[test]
fn leak_mutually_adopted() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    let s = "a".repeat(1024 * 1024);

    leak::Detector::new("mutually adopted", None, None).check_leaks(|_| {
        let first = Rc::new(s.clone());
        let last = Rc::new(s.clone());
        unsafe {
            Rc::adopt(&first, &last);
            Rc::adopt(&last, &first);
        }
        drop(first);
        drop(last);
    });
}
