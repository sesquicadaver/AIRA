//! Living spec smoke (#121): C0 case ids in `implementation-status.md` ↔ `run_c0`.

use std::collections::BTreeSet;
use std::path::PathBuf;

use aira_conformance::run_c0;
use tempfile::tempdir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Canonical C0 case list lives in the Conformance profiles table row for C0.
fn c0_case_ids_from_implementation_status() -> BTreeSet<String> {
    let text = std::fs::read_to_string(repo_root().join("docs/implementation-status.md"))
        .expect("read implementation-status.md");
    let row = text
        .lines()
        .find(|line| line.starts_with("| C0 |") && line.contains("run_c0"))
        .expect("Conformance profiles table missing C0 row with run_c0");
    extract_c0_ids_from_row(row)
}

fn extract_c0_ids_from_row(row: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for part in row.split('`') {
        if part.starts_with("c0.") {
            ids.insert(part.to_string());
        }
    }
    assert!(
        !ids.is_empty(),
        "C0 conformance row must list backtick-wrapped c0.* case ids"
    );
    ids
}

fn c0_case_ids_from_runner() -> BTreeSet<String> {
    let dir = tempdir().expect("tempdir");
    let suite = run_c0(dir.path()).expect("run_c0");
    suite.cases.iter().map(|c| c.test_id.clone()).collect()
}

#[test]
fn living_spec_c0_ids_in_doc_exist_in_run_c0() {
    let doc_ids = c0_case_ids_from_implementation_status();
    let runner_ids = c0_case_ids_from_runner();
    for id in &doc_ids {
        assert!(
            runner_ids.contains(id),
            "implementation-status lists C0 case `{id}` but run_c0 has no such test_id"
        );
    }
}

#[test]
fn living_spec_run_c0_ids_documented_in_implementation_status() {
    let doc_ids = c0_case_ids_from_implementation_status();
    let runner_ids = c0_case_ids_from_runner();
    for id in &runner_ids {
        assert!(
            doc_ids.contains(id),
            "run_c0 case `{id}` missing from C0 row in implementation-status.md"
        );
    }
}

#[test]
fn living_spec_missing_doc_case_would_fail() {
    let row = "| C0 | Local Core | `run_c0` | `c0.ontology.schemas`, `c0.fake.missing` |";
    let ids = extract_c0_ids_from_row(row);
    let runner_ids = c0_case_ids_from_runner();
    assert!(!runner_ids.contains("c0.fake.missing"));
    assert!(ids.contains("c0.fake.missing"));
}
