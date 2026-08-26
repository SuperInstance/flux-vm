//! Runs the conformance harness and writes CONFORMANCE.md at the repo root.

use flux_conformance::{differ, report};

fn main() {
    let reports = differ::evaluate_corpus();
    let summary = differ::Summary::from_reports(&reports);
    let md = report::generate(&reports);

    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("CONFORMANCE.md");
    std::fs::write(&out, &md).expect("write CONFORMANCE.md");

    println!("wrote {}", out.display());
    println!(
        "cases={} critical={} major={} minor={} info={} cases_with_divergence={}",
        summary.cases,
        summary.critical,
        summary.major,
        summary.minor,
        summary.info,
        summary.cases_with_divergence
    );
}
