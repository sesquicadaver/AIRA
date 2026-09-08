//! Phase Q contract smoke (#275 wiring … #285 RFC-0164 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_q_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-q-plan.md")).unwrap();
    for needle in [
        "Phase Q",
        "#275",
        "#285",
        "cross-install",
        "light monitoring",
        "Unique Desktop identity",
        "Model light observe",
        "Model verify context",
        "Reachability observation independence",
        "Reachability NAT endpoints",
        "Reachability evidence admission",
        "Lifecycle revision races",
        "Backend≠model used",
        "Invite import atomicity",
        "AIRA-RFC-0164",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE P closed",
        "QUEUE Q closed",
        "no OPEN Q atoms",
        "GPU marketplace",
        "Calculate 2 + 2",
        "**DONE** @",
        "RFC-0173",
        "00f19cf",
    ] {
        assert!(text.contains(needle), "phase-q-plan missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#285`"),
        "phase-q-plan must not keep #285 as first-OPEN after close"
    );
}

#[test]
fn phase_q_queue_all_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-q-plan.md"));
    for n in 275..=285 {
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
        "Analyze-320",
        "RFC-0164",
        "RFC-0173",
        "QUEUE Q closed",
        "no OPEN Q atoms",
        "QUEUE P closed",
        "desktop-ux.md",
        "aira-current.md",
        "00f19cf",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#285`"),
        "QUEUE must not keep #285 as first-OPEN after close"
    );
}

#[test]
fn phase_q_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-q-plan.md",
        "#285",
        "RFC-0164",
        "QUEUE Q closed",
        "cross-install",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0164_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0164-phase-q-cross-install-integrity.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0164 missing");
    for needle in ["#285", "QUEUE Q closed", "no OPEN Q atoms", "RFC-0173"] {
        assert!(text.contains(needle), "RFC-0164 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0173_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0173-invite-import-atomicity.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0173 missing");
    for needle in [
        "#284",
        "TrustStore",
        "AddressBook",
        "validate_aira_bind",
        "rollback",
    ] {
        assert!(text.contains(needle), "RFC-0173 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0172_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0172-backend-ne-model-used.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0172 missing");
    for needle in [
        "#283",
        "backend:",
        "model_ref",
        "used",
        "extract_used_model",
    ] {
        assert!(text.contains(needle), "RFC-0172 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0171_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0171-lifecycle-revision-races.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0171 missing");
    for needle in [
        "#282",
        "lifecycle_revision",
        "Quit",
        "used_settings",
        "refresh",
    ] {
        assert!(text.contains(needle), "RFC-0171 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0170_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0170-reachability-evidence-admission.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0170 missing");
    for needle in [
        "#281",
        "root",
        "replay",
        "apply-time",
        "REACHABILITY_APPLY_MAX_SKEW_SECS",
    ] {
        assert!(text.contains(needle), "RFC-0170 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0169_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0169-reachability-nat-endpoints.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0169 missing");
    for needle in [
        "#280",
        "local_addr",
        "export_from_session",
        "NAT",
        "Outbound",
    ] {
        assert!(text.contains(needle), "RFC-0169 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0168_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0168-reachability-observation-independence.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0168 missing");
    for needle in [
        "#279",
        "local_checked_at",
        "external_checked_at",
        "mark_local_bind",
        "skew",
    ] {
        assert!(text.contains(needle), "RFC-0168 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0167_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0167-model-verify-context.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0167 missing");
    for needle in [
        "#278",
        "load_node_identity",
        "root-scoped",
        "verify_ed25519",
        "reopen",
    ] {
        assert!(text.contains(needle), "RFC-0167 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0166_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0166-model-light-observe.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0166 missing");
    for needle in [
        "#277",
        "ObserveLight",
        "observe-ready",
        "AdmitFull",
        "check_activated",
    ] {
        assert!(text.contains(needle), "RFC-0166 missing: {needle}");
    }
}

#[test]
fn phase_q_rfc_0165_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0165-unique-desktop-identity.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0165 missing");
    for needle in ["#276", "desktop.", "KeyCollision", "ensure_bootstrap"] {
        assert!(text.contains(needle), "RFC-0165 missing: {needle}");
    }
}

#[test]
fn phase_q_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-q-plan.md") || readme.contains("Phase Q"));
    assert!(readme.contains("QUEUE Q closed") || readme.contains("RFC-0164"));
    assert!(
        !readme.contains("first OPEN `#285`") && !readme.contains("first OPEN `#268`"),
        "README must not keep stale first-OPEN tips"
    );
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-q-plan.md"));
    assert!(docs.contains("QUEUE Q closed") || docs.contains("RFC-0164"));
}

#[test]
fn phase_q_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(
        text.contains("phase-q-plan.md")
            || text.contains("Phase Q")
            || text.contains("QUEUE Q closed")
    );
    assert!(text.contains("немає OPEN") || text.contains("QUEUE Q closed"));
    assert!(
        !text.contains("перший OPEN `#285`") && !text.contains("first OPEN `#285`"),
        "NEXT_PROBLEM must not keep #285 as first-OPEN"
    );
}

#[test]
fn phase_q_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #285 |") || status.contains("#285"));
    assert!(status.contains("phase_q_doc.rs"));
    assert!(status.contains("phase-q-plan.md"));
    assert!(status.contains("RFC-0164") || status.contains("0164"));
    assert!(status.contains("QUEUE Q closed") || status.contains("DONE @ RFC-0164"));
}

#[test]
fn phase_p_points_to_phase_q() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-p-plan.md")).unwrap();
    assert!(
        text.contains("phase-q-plan.md") || text.contains("Phase Q"),
        "phase-p-plan should point at Phase Q after activation"
    );
}
