//! Phase X contract smoke (#343 wiring … #358 RFC-0226 close).

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_x_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-x-plan.md")).unwrap();
    for needle in [
        "Phase X",
        "Pack 2",
        "local multi-model GUI",
        "#343",
        "#358",
        "IN PROGRESS",
        "Per-model inventory",
        "Model select API",
        "Settings Models",
        "Work Compare",
        "Two-model installed acceptance",
        "AIRA-RFC-0226",
        "confirmed free",
        "QUEUE W closed",
        "RFC-0215",
        "RFC-0226",
        "RFC-0227",
        "RFC-0228",
        "RFC-0229",
        "RFC-0230",
        "RFC-0231",
        "first OPEN `#349`",
        "M1",
        "M6",
        "GPU marketplace",
        "aira-core",
        "public bind",
        "phase-w-plan.md",
    ] {
        assert!(text.contains(needle), "phase-x-plan missing: {needle}");
    }
    assert!(
        !text.contains("**QUEUED** (записано") || text.contains("**IN PROGRESS**"),
        "phase-x-plan must be activated (IN PROGRESS) after #343"
    );
    assert!(
        !text.contains("**DONE** @ [AIRA-RFC-0226"),
        "phase-x-plan must not claim RFC-0226 DONE before #358"
    );
    assert!(
        !text.contains("first OPEN `#343`")
            && !text.contains("first OPEN `#344`")
            && !text.contains("first OPEN `#345`")
            && !text.contains("first OPEN `#346`")
            && !text.contains("first OPEN `#347`")
            && !text.contains("first OPEN `#348`"),
        "phase-x-plan must advance tip past #348"
    );
}

#[test]
fn phase_x_queue_348_done_349_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-x-plan.md"));
    for n in 343..=348 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(text.contains("| 349 | **OPEN**"), "QUEUE #349 must be OPEN");
    for n in 349..=358 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 348 | **OPEN**"),
        "QUEUE #348 must not stay OPEN"
    );
    for needle in [
        "Analyze-380",
        "Analyze-381",
        "Analyze-382",
        "Analyze-383",
        "Analyze-384",
        "Analyze-385",
        "RFC-0226",
        "RFC-0227",
        "RFC-0228",
        "RFC-0229",
        "RFC-0230",
        "RFC-0231",
        "RFC-0215",
        "QUEUE W closed",
        "Pack 2",
        "first OPEN `#348`",
        "first OPEN `#349`",
        "phase_x_doc",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        text.contains("**Перший OPEN:** `#349`"),
        "QUEUE tip must be first OPEN #349 after #348"
    );
    assert!(
        !text.contains("**Перший OPEN:** `#348`"),
        "QUEUE tip must not keep #348 as first-OPEN"
    );
}

#[test]
fn phase_x_rfc_0226_file_free() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0226-phase-x-pack2-multi-model-gui.md");
    assert!(
        !path.exists(),
        "RFC-0226 must stay file-free until #358 close"
    );
}

#[test]
fn phase_x_rfc_0227_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0227-per-model-inventory-lifecycle.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0227 missing after #344");
    for needle in [
        "#344",
        "list_model_lifecycle",
        "activated.latest",
        "model_artifact_ref",
        "RFC-0226",
    ] {
        assert!(text.contains(needle), "RFC-0227 missing: {needle}");
    }
}

#[test]
fn phase_x_rfc_0228_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0228-model-select-api.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0228 missing after #345");
    for needle in [
        "#345",
        "select_model",
        "ModelSelection",
        "NoAvailableModels",
        "ModelUnready",
        "ModelRemoved",
        "first OPEN #346",
        "RFC-0227",
    ] {
        assert!(text.contains(needle), "RFC-0228 missing: {needle}");
    }
}

#[test]
fn phase_x_rfc_0229_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0229-request-profile-snapshot.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0229 missing after #346");
    for needle in [
        "#346",
        "excluded_model_refs",
        "freeze",
        "first OPEN #347",
        "RFC-0228",
    ] {
        assert!(text.contains(needle), "RFC-0229 missing: {needle}");
    }
}

#[test]
fn phase_x_rfc_0230_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0230-cli-supported-contract-parity.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0230 missing after #347");
    for needle in [
        "#347",
        "excluded-model-ref",
        "no-op",
        "first OPEN #348",
        "RFC-0229",
    ] {
        assert!(text.contains(needle), "RFC-0230 missing: {needle}");
    }
}

#[test]
fn phase_x_rfc_0231_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0231-settings-models-catalog-gui.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0231 missing after #348");
    for needle in ["#348", "Settings", "catalog", "first OPEN #349", "RFC-0230"] {
        assert!(text.contains(needle), "RFC-0231 missing: {needle}");
    }
}

#[test]
fn phase_x_settings_models_catalog_runtime_present() {
    let cat = std::fs::read_to_string(
        repo_root().join("crates/aira-desktop-runtime/src/model_catalog.rs"),
    )
    .unwrap();
    assert!(cat.contains("load_model_catalog"));
    assert!(cat.contains("scan_model_catalog"));
    assert!(cat.contains("add_model_file"));
    assert!(cat.contains("prepare_model"));
    assert!(cat.contains("select_catalog_model"));
    assert!(cat.contains("#348") || cat.contains("RFC-0231"));
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(ui.contains("settings_models_catalog_hint") || ui.contains("models_catalog_scan"));
    assert!(!ui.contains("settings_models_observe_only"));
    let actions =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/actions.rs")).unwrap();
    assert!(actions.contains("models_catalog_load"));
}

#[test]
fn phase_x_inventory_lifecycle_runtime_present() {
    let life = std::fs::read_to_string(repo_root().join("csu/model-acquisition/src/lifecycle.rs"))
        .unwrap();
    assert!(life.contains("list_model_lifecycle"));
    assert!(life.contains("#344") || life.contains("RFC-0227"));
    let activate =
        std::fs::read_to_string(repo_root().join("csu/model-acquisition/src/activate.rs")).unwrap();
    assert!(activate.contains("activate_verified_model"));
    assert!(activate.contains("write_activated_slot"));
    let gate =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/activate_gate.rs")).unwrap();
    assert!(gate.contains("resolve_admit_pointer"));
    assert!(gate.contains("ACTIVATED_SLOT_POINTER_NAME"));
    let lib =
        std::fs::read_to_string(repo_root().join("csu/model-acquisition/src/lib.rs")).unwrap();
    assert!(lib.contains("two_models_verified_and_available_independently_after_latest_moves"));
}

#[test]
fn phase_x_select_api_runtime_present() {
    let sel =
        std::fs::read_to_string(repo_root().join("csu/model-acquisition/src/select.rs")).unwrap();
    assert!(sel.contains("select_model"));
    assert!(sel.contains("ModelSelection"));
    assert!(sel.contains("#345") || sel.contains("RFC-0228"));
    assert!(sel.contains("auto_empty_is_explained"));
    assert!(sel.contains("required_unready_is_explained"));
    assert!(sel.contains("required_does_not_silent_substitute_other_available"));
}

#[test]
fn phase_x_profile_snapshot_runtime_present() {
    let admission =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/admission.rs")).unwrap();
    assert!(admission.contains("excluded_model_refs"));
    assert!(admission.contains("freeze_auto_within_set"));
    assert!(admission.contains("#346") || admission.contains("RFC-0229"));
    let sel =
        std::fs::read_to_string(repo_root().join("csu/model-acquisition/src/select.rs")).unwrap();
    assert!(sel.contains("select_model_with_profile"));
    assert!(sel.contains("ModelSelectProfile"));
    let lib = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    assert!(lib.contains("submit_with_admission_keeps_snapshot_after_settings_like_mutation"));
}

#[test]
fn phase_x_cli_parity_runtime_present() {
    let cli = std::fs::read_to_string(repo_root().join("crates/aira-cli/src/cli.rs")).unwrap();
    assert!(cli.contains("excluded-model-ref") || cli.contains("excluded_model_refs"));
    assert!(cli.contains("#347") || cli.contains("RFC-0230"));
    assert!(!cli.contains("temperature: Option"));
    assert!(!cli.contains("privacy_class: Option"));
    assert!(!cli.contains("allow_model_fallback: bool"));
    let problem =
        std::fs::read_to_string(repo_root().join("crates/aira-cli/src/commands/problem.rs"))
            .unwrap();
    assert!(problem.contains("constraints_from_submit_flags"));
    assert!(problem.contains("excluded_model_refs"));
    assert!(problem.contains("allowed_minus_excluded_freezes_into_snapshot"));
    let main = std::fs::read_to_string(repo_root().join("crates/aira-cli/src/main.rs")).unwrap();
    assert!(main.contains("rejects_removed_temperature_flag"));
    assert!(main.contains("parses_allowed_and_excluded_model_refs"));
}

#[test]
fn phase_w_still_closed_canon() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-w-plan.md")).unwrap();
    assert!(
        text.contains("QUEUE W closed") || text.contains("RFC-0215"),
        "phase-w-plan should remain closed canon"
    );
    assert!(
        text.contains("**DONE** @ [AIRA-RFC-0215"),
        "Phase W must stay DONE @ RFC-0215"
    );
}

#[test]
fn phase_x_desktop_ux_tip() {
    let text = std::fs::read_to_string(repo_root().join("docs/desktop-ux.md")).unwrap();
    for needle in [
        "phase-x-plan.md",
        "#345",
        "#346",
        "#347",
        "#348",
        "#349",
        "RFC-0226",
        "QUEUE W closed",
    ] {
        assert!(text.contains(needle), "desktop-ux missing: {needle}");
    }
}

#[test]
fn phase_x_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-x-plan.md") || readme.contains("Phase X"));
    assert!(readme.contains("#349") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-x-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("#349"));
}

#[test]
fn phase_x_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-x-plan.md") || text.contains("Phase X"));
    assert!(text.contains("#349") || text.contains("перший OPEN"));
    assert!(text.contains("QUEUE W closed") || text.contains("RFC-0215"));
}

#[test]
fn phase_x_status_row() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("| #345 |") || status.contains("#345"));
    assert!(status.contains("phase_x_doc.rs"));
    assert!(status.contains("phase-x-plan.md"));
    assert!(status.contains("RFC-0228") || status.contains("0228"));
    assert!(status.contains("RFC-0229") || status.contains("0229"));
    assert!(status.contains("RFC-0230") || status.contains("0230"));
    assert!(status.contains("#346"));
    assert!(status.contains("#347"));
    assert!(status.contains("#348"));
    assert!(status.contains("#349"));
    assert!(status.contains("RFC-0231") || status.contains("0231"));
}
