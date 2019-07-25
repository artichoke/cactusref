#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings, intra_doc_link_resolution_failure)]

use cactusref::{Adoptable, Rc};

mod leak;

#[test]
fn leak_adopt_with_dropped_rc() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    let s = "a".repeat(1024 * 1024);

    leak::Detector::new("adopt with dropped Rc", None, None).check_leaks(|_| {
        let first = Rc::new(s.clone());
        let mut last = Rc::clone(&first);
        for _ in 1..10 {
            let obj = Rc::new(s.clone());
            Rc::adopt(&obj, &last);
            last = obj;
        }
        Rc::adopt(&first, &last);
        drop(first);
        drop(last);
    });
}
