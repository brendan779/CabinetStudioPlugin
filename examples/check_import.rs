//! Checks opening a Cabinet Studio job (`spec::import_job`) builds every item through the core, on a
//! job file of Cabinet Studio's own tests, and on any `.cabinetjob` paths given.
//!
//!     cargo run -q --example check_import --target aarch64-apple-darwin -- [more.cabinetjob ...]
#[path = "../src/spec.rs"]
#[allow(dead_code)]
mod spec;

use cabinet_core::run::{Run, build_run};

fn check(name: &str, text: &str) {
    let (job_value, items) = spec::import_job(text).unwrap_or_else(|e| panic!("{name}: {e}"));
    let job = spec::Job::from_stored(&job_value);
    let run: Vec<_> = items
        .iter()
        .map(|i| spec::item(i, &job, "").unwrap_or_else(|| panic!("{name}: can't read {}", i["id"])))
        .collect();
    let n = run.len();
    let lists = build_run(&Run { origin: job.origin, items: run }).unwrap_or_else(|e| panic!("{name}: {e}"));
    let parts: usize = lists.iter().map(|l| l.parts.len()).sum();
    assert!(parts > 0, "{name}: no parts");
    println!("{name}: {n} items, {parts} parts");
}

fn main() {
    check("wardrobe-runners", include_str!("fixtures/wardrobe-runners.cabinetjob"));
    for path in std::env::args().skip(1) {
        check(&path, &std::fs::read_to_string(&path).expect("readable"));
    }
}
