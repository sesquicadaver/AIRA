//! Phase J plan is PLANNED only: not in QUEUE, RFC-0096 file must not exist yet.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_j_plan_planned_not_queued() {
    let plan = std::fs::read_to_string(repo_root().join("docs/phase-j-plan.md")).unwrap();
    for needle in [
        "Phase J",
        "**PLANNED**",
        "#199",
        "#208",
        "RFC-0096",
        "confirmed free",
        "object_store_access",
        "B1-010",
        "Book II",
        "GPU marketplace",
        "не в QUEUE",
    ] {
        assert!(plan.contains(needle), "phase-j-plan missing: {needle}");
    }

    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        queue.contains("phase-j-plan.md"),
        "QUEUE must index the Phase J plan"
    );
    assert!(
        !queue.contains("| 199 |"),
        "QUEUE must not grow #199 rows until wiring"
    );
    assert!(
        !queue.contains("| 199 | **OPEN**"),
        "Phase J must not be OPEN"
    );

    let rfc_dir = repo_root().join("specs/rfc");
    for entry in std::fs::read_dir(&rfc_dir).expect("specs/rfc") {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            !name.starts_with("AIRA-RFC-0096"),
            "RFC-0096 reserved for J close; unexpected file: {name}"
        );
    }

    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-j-plan.md"));
    assert!(docs.contains("**PLANNED**"));
}
