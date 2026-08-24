//! CI governance doc contract (#109, #120).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const REQUIRED_MERGE_CHECKS: &[&str] = &["fmt-clippy-test-schema-c0-c1", "conformance-c2"];

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
        "conformance-c2",
        "conformance run --profile C2",
        "1.94.0",
        "main",
        "develop",
        "AIRA-RFC-0058",
        "AIRA-RFC-0070",
        "branch protection",
        "Branch protection checklist",
        "ci_governance_doc.rs",
    ] {
        assert!(text.contains(needle), "doc missing: {needle}");
    }
}

#[test]
fn required_merge_checks_documented_and_in_workflow() {
    let doc = std::fs::read_to_string(repo_root().join("docs/ci-governance.md")).unwrap();
    let ci = std::fs::read_to_string(repo_root().join(".github/workflows/ci.yml")).unwrap();

    for check in REQUIRED_MERGE_CHECKS {
        assert!(
            doc.contains(check),
            "ci-governance.md missing required check: {check}"
        );
        assert!(
            ci.contains(&format!("name: {check}")),
            "ci.yml missing job name: {check}"
        );
    }

    assert!(
        doc.contains("select `fmt-clippy-test-schema-c0-c1` and `conformance-c2`"),
        "branch protection step must list both required checks"
    );
}

#[test]
fn ci_workflow_c2_is_separate_job() {
    let ci = std::fs::read_to_string(repo_root().join(".github/workflows/ci.yml")).unwrap();
    assert!(ci.contains("conformance-c2:"));
    assert!(ci.contains("name: conformance-c2"));
    assert!(ci.contains("conformance run --profile C2"));
    // C2 must not run inside the primary check job (regression guard for #117 split).
    let check_block = ci.split("conformance-c2:").next().expect("check job block");
    assert!(
        !check_block.contains("conformance run --profile C2"),
        "C2 must be a separate job, not a step in check"
    );
}

#[test]
fn ci_workflow_matches_governance_job_name() {
    let doc = std::fs::read_to_string(repo_root().join("docs/ci-governance.md")).unwrap();
    let ci = std::fs::read_to_string(repo_root().join(".github/workflows/ci.yml")).unwrap();
    assert!(doc.contains("fmt-clippy-test-schema-c0-c1"));
    assert!(ci.contains("name: fmt-clippy-test-schema-c0-c1"));
    assert!(ci.contains("name: conformance-c2"));
}
