//! Phase O contract smoke (#255 wiring … #265 RFC-0146 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_o_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-o-plan.md")).unwrap();
    for needle in [
        "Phase O",
        "#255",
        "#265",
        "SystemSnapshot",
        "F1",
        "AIRA-RFC-0146",
        "AIRA-RFC-0151",
        "AIRA-RFC-0152",
        "AIRA-RFC-0153",
        "AIRA-RFC-0154",
        "AIRA-RFC-0155",
        "confirmed free",
        "desktop-ux.md",
        "QUEUE N-fix closed",
        "QUEUE O closed",
        "GPU marketplace",
        "Calculate 2 + 2",
        "**DONE** @",
        "no OPEN O atoms",
    ] {
        assert!(text.contains(needle), "phase-o-plan missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#265`"),
        "phase-o-plan must not keep #265 as first-OPEN after close"
    );
}

#[test]
fn phase_o_queue_all_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-o-plan.md"));
    for n in 255..=265 {
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
        "Analyze-300",
        "RFC-0146",
        "RFC-0155",
        "QUEUE O closed",
        "no OPEN O atoms",
        "desktop-ux.md",
        "QUEUE N-fix closed",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        !text.contains("first OPEN `#265`"),
        "QUEUE must not keep #265 as first-OPEN after close"
    );
}

#[test]
fn phase_o_desktop_ux_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-o-plan.md",
        "Робота",
        "Стан системи",
        "Параметри",
        "Довідка",
        "F1",
        "SystemSnapshot",
        "UNKNOWN",
        "AddressBook",
        "help_id",
        "lexicon",
        "#265",
        "RFC-0146",
        "help:",
        "Restart",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_o_rfc_0151_present() {
    assert!(repo_root()
        .join("specs/rfc/AIRA-RFC-0151-work-screen-provenance.md")
        .is_file());
}

#[test]
fn phase_o_rfc_0152_present() {
    assert!(repo_root()
        .join("specs/rfc/AIRA-RFC-0152-system-status-screen.md")
        .is_file());
}

#[test]
fn phase_o_rfc_0153_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0153-settings-apply-lifecycle.md");
    assert!(path.is_file(), "RFC-0153 missing");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("#262"));
    assert!(text.contains("Restart") || text.contains("Applied"));
}

#[test]
fn phase_o_rfc_0154_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0154-offline-f1-shell.md");
    assert!(path.is_file(), "RFC-0154 missing");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("#263"));
    assert!(text.contains("include_str") || text.contains("embed"));
    assert!(text.contains("search"));
}

#[test]
fn phase_o_rfc_0155_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0155-seed-help-topics.md");
    assert!(path.is_file(), "RFC-0155 missing");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("#264"));
    assert!(text.contains("help:"));
    assert!(text.contains("link"));
}

#[test]
fn phase_o_rfc_0146_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0146-phase-o-desktop-ux-help.md");
    assert!(path.is_file(), "RFC-0146 missing");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("#265"));
    assert!(text.contains("QUEUE O closed"));
    assert!(text.contains("RFC-0155"));
    assert!(text.contains("confirmed free"));
    assert!(text.contains("no OPEN O atoms") || text.contains("QUEUE O closed"));
}

#[test]
fn phase_o_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-o-plan.md"));
    assert!(readme.contains("QUEUE O closed") || readme.contains("RFC-0146"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-o-plan.md"));
    assert!(docs.contains("QUEUE O closed") || docs.contains("RFC-0146"));
}

#[test]
fn phase_n_fix_points_to_phase_o() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-n-fix-plan.md")).unwrap();
    assert!(text.contains("phase-o-plan.md"));
}

#[test]
fn phase_o_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-o-plan.md") || text.contains("Phase O"));
    assert!(text.contains("QUEUE N-fix closed") || text.contains("no OPEN N-fix"));
    assert!(text.contains("QUEUE O closed") || text.contains("RFC-0146"));
    assert!(
        !text.contains("перший OPEN `#265`") && !text.contains("first OPEN `#265`"),
        "NEXT_PROBLEM must not keep #265 as first-OPEN"
    );
}

#[test]
fn phase_o_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #265 |") || status.contains("#265"));
    assert!(status.contains("RFC-0146") || status.contains("0146"));
    assert!(status.contains("QUEUE O closed") || status.contains("DONE @ RFC-0146"));
    assert!(status.contains("phase_o_doc.rs"));
}

#[test]
fn phase_o_help_seed_catalog_and_readme() {
    let root = repo_root().join("docs/help");
    assert!(
        root.join("README.md").is_file(),
        "docs/help/README.md missing"
    );
    for lang in ["en", "uk"] {
        for name in [
            "start.md",
            "work.submit.md",
            "work.result.md",
            "work.waiting.md",
            "model.select.md",
            "model.unavailable.md",
            "network.connect.md",
            "network.reachability.md",
            "network.trust.md",
            "settings.apply.md",
            "node.lifecycle.md",
        ] {
            let path = root.join(lang).join(name);
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
            assert!(
                text.contains("help:"),
                "{} must contain help: related links",
                path.display()
            );
            assert!(
                text.len() >= 500,
                "{} seed too short ({})",
                path.display(),
                text.len()
            );
        }
    }
}
