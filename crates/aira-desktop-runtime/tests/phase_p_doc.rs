//! Phase P contract smoke (#266 wiring … #274 RFC-0156 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_p_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-p-plan.md")).unwrap();
    for needle in [
        "Phase P",
        "#266",
        "#274",
        "runtime-honest",
        "DataQuality",
        "Applied",
        "Model triple",
        "Reachability",
        "AIRA-RFC-0156",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE O closed",
        "QUEUE P closed",
        "no OPEN P atoms",
        "GPU marketplace",
        "Calculate 2 + 2",
        "**DONE** @",
        "RFC-0163",
    ] {
        assert!(text.contains(needle), "phase-p-plan missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#274`"),
        "phase-p-plan must not keep #274 as first-OPEN after close"
    );
}

#[test]
fn phase_p_queue_all_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-p-plan.md"));
    for n in 266..=274 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
        assert!(
            !text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must not stay OPEN"
        );
    }
    for needle in [
        "Analyze-309",
        "RFC-0156",
        "RFC-0163",
        "QUEUE P closed",
        "no OPEN P atoms",
        "desktop-ux.md",
        "aira-current.md",
        "QUEUE O closed",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#274`"),
        "QUEUE must not keep #274 as first-OPEN after close"
    );
}

#[test]
fn phase_p_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-p-plan.md",
        "#274",
        "RFC-0156",
        "QUEUE P closed",
        "runtime-honest",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_p_rfc_0156_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0156-phase-p-runtime-honest-desktop.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0156 missing");
    for needle in ["#274", "QUEUE P closed", "no OPEN P atoms", "RFC-0163"] {
        assert!(text.contains(needle), "RFC-0156 missing: {needle}");
    }
}

#[test]
fn phase_p_rfc_0162_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0162-lifecycle-nonblocking.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0162 missing");
    for needle in ["#272", "update()", "refresh", "Start"] {
        assert!(text.contains(needle), "RFC-0162 missing: {needle}");
    }
}

#[test]
fn phase_p_rfc_0163_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0163-help-f1-routing.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0163 missing");
    for needle in ["#273", "help_search", "focus", "last_problem"] {
        assert!(text.contains(needle), "RFC-0163 missing: {needle}");
    }
}

#[test]
fn phase_p_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-p-plan.md"));
    assert!(readme.contains("QUEUE P closed") || readme.contains("RFC-0156"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-p-plan.md"));
    assert!(docs.contains("QUEUE P closed") || docs.contains("RFC-0156"));
}

#[test]
fn phase_o_points_to_phase_p() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-o-plan.md")).unwrap();
    assert!(text.contains("phase-p-plan.md") || text.contains("Phase P"));
}

#[test]
fn phase_p_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-p-plan.md") || text.contains("Phase P") || text.contains("QUEUE P closed"));
    assert!(text.contains("немає OPEN") || text.contains("QUEUE P closed"));
    assert!(
        !text.contains("перший OPEN `#274`") && !text.contains("first OPEN `#274`"),
        "NEXT_PROBLEM must not keep #274 as first-OPEN"
    );
}

#[test]
fn phase_p_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #274 |") || status.contains("#274"));
    assert!(status.contains("RFC-0156") || status.contains("0156"));
    assert!(status.contains("QUEUE P closed") || status.contains("DONE @ RFC-0156"));
    assert!(status.contains("phase_p_doc.rs"));
    assert!(status.contains("phase-p-plan.md"));
}
