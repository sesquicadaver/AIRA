//! CI governance doc contract (#109).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn ci_governance_doc_contract() {
    let path = repo_root().join("docs/ci-governance.md");
    let text = std::fs::read_to_string(&path).expect("read ci-governance.md");
    for needle in [
        "fmt-clippy-test-schema-c0-c1",
        ".github/workflows/ci.yml",
        "dependency firewall",
        "schema validate",
        "conformance run --profile C0",
        "conformance run --profile C1",
        "1.94.0",
        "main",
        "develop",
        "AIRA-RFC-0058",
        "branch protection",
    ] {
        assert!(text.contains(needle), "doc missing: {needle}");
    }
}

#[test]
fn ci_workflow_matches_governance_job_name() {
    let doc = std::fs::read_to_string(repo_root().join("docs/ci-governance.md")).unwrap();
    let ci = std::fs::read_to_string(repo_root().join(".github/workflows/ci.yml")).unwrap();
    assert!(doc.contains("fmt-clippy-test-schema-c0-c1"));
    assert!(ci.contains("name: fmt-clippy-test-schema-c0-c1"));
}
