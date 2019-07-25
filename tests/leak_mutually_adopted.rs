#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings, intra_doc_link_resolution_failure)]

use cactusref::{Adoptable, CactusRef};

mod leak;

#[test]
fn leak_mutually_adopted() {
    env_logger::Builder::from_env("CACTUS_LOG").init();

    let s = "a".repeat(1024 * 1024);

    leak::Detector::new("mutually adopted", None, None).check_leaks(|_| {
        let first = CactusRef::new(s.clone());
        let last = CactusRef::new(s.clone());
        CactusRef::adopt(&first, &last);
        CactusRef::adopt(&last, &first);
        drop(first);
        drop(last);
    });
}
