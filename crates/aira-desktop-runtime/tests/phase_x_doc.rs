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
        "RFC-0232",
        "RFC-0233",
        "RFC-0234",
        "RFC-0235",
        "RFC-0236",
        "RFC-0237",
        "RFC-0238",
        "first OPEN `#356`",
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
            && !text.contains("first OPEN `#348`")
            && !text.contains("first OPEN `#349`")
            && !text.contains("first OPEN `#350`")
            && !text.contains("first OPEN `#351`")
            && !text.contains("first OPEN `#352`")
            && !text.contains("first OPEN `#353`")
            && !text.contains("first OPEN `#354`")
            && !text.contains("first OPEN `#355`"),
        "phase-x-plan must advance tip past #355"
    );
}

#[test]
fn phase_x_queue_355_done_356_open() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(text.contains("phase-x-plan.md"));
    for n in 343..=355 {
        assert!(
            text.contains(&format!("| {n} | **DONE**")),
            "QUEUE #{n} must be DONE"
        );
    }
    assert!(text.contains("| 356 | **OPEN**"), "QUEUE #356 must be OPEN");
    for n in 356..=358 {
        assert!(
            text.contains(&format!("| {n} | **OPEN**")),
            "QUEUE #{n} must be OPEN"
        );
    }
    assert!(
        !text.contains("| 355 | **OPEN**"),
        "QUEUE #355 must not stay OPEN"
    );
    for needle in [
        "Analyze-380",
        "Analyze-381",
        "Analyze-382",
        "Analyze-383",
        "Analyze-384",
        "Analyze-385",
        "Analyze-386",
        "Analyze-387",
        "Analyze-388",
        "Analyze-389",
        "Analyze-390",
        "Analyze-391",
        "Analyze-392",
        "RFC-0226",
        "RFC-0227",
        "RFC-0228",
        "RFC-0229",
        "RFC-0230",
        "RFC-0231",
        "RFC-0232",
        "RFC-0233",
        "RFC-0234",
        "RFC-0235",
        "RFC-0236",
        "RFC-0237",
        "RFC-0238",
        "RFC-0215",
        "QUEUE W closed",
        "Pack 2",
        "first OPEN `#353`",
        "first OPEN `#354`",
        "first OPEN `#355`",
        "first OPEN `#356`",
        "phase_x_doc",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
    assert!(
        text.contains("**Перший OPEN:** `#356`"),
        "QUEUE tip must be first OPEN #356 after #355"
    );
    assert!(
        !text.contains("**Перший OPEN:** `#355`"),
        "QUEUE tip must not keep #355 as first-OPEN"
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
fn phase_x_rfc_0234_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0234-network-address-honesty-p-ux.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0234 missing after #351");
    for needle in [
        "#351",
        "Network/address honesty",
        "loopback",
        "advertised",
        "P3",
        "P4",
        "first OPEN #352",
    ] {
        assert!(text.contains(needle), "RFC-0234 missing: {needle}");
    }
}

#[test]
fn phase_x_network_address_honesty_runtime_present() {
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(ui.contains("peer_listen_loopback_hint"));
    assert!(ui.contains("addr_roles_hint"));
    assert!(ui.contains("p34_base") || ui.contains("p34_mutex_hint"));
    assert!(ui.contains("addr_http") || ui.contains("addr_peer_listen"));
    assert!(ui.contains("selectable_label") && ui.contains("NetworkProfile::P3"));
    let i18n =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    assert!(i18n.contains("peer_listen_loopback_hint"));
    assert!(i18n.contains("addr_advertised"));
    assert!(i18n.contains("Local only (P0)") || i18n.contains("Лише локально (P0)"));
    let help_en =
        std::fs::read_to_string(repo_root().join("docs/help/en/network.connect.md")).unwrap();
    assert!(help_en.contains("Three address roles") || help_en.contains("HTTP listen"));
    assert!(help_en.contains("P3 | P4") || help_en.contains("mutually exclusive"));
}

#[test]
fn phase_x_rfc_0235_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0235-human-copy-settings-system-ia.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0235 missing after #352");
    for needle in [
        "#352",
        "Human copy",
        "Settings",
        "Stop node",
        "first OPEN #353",
    ] {
        assert!(text.contains(needle), "RFC-0235 missing: {needle}");
    }
}

#[test]
fn phase_x_human_copy_settings_ia_runtime_present() {
    let i18n =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/i18n.rs")).unwrap();
    assert!(i18n.contains("Stop node") || i18n.contains("Зупинити вузол"));
    assert!(i18n.contains("Exit AIRA") || i18n.contains("Завершити AIRA"));
    assert!(i18n.contains("open_settings_connection"));
    assert!(i18n.contains("settings_connection_edit_hint"));
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(ui.contains("open_settings_connection"));
    assert!(ui.contains("settings_connection_edit_hint"));
    assert!(ui.contains("sys_connection_observe_hint"));
    // Settings Connection owns profile selectable_label edit
    let settings_idx = ui.find("fn ui_settings").expect("ui_settings");
    let settings = &ui[settings_idx..];
    assert!(
        settings.contains("selectable_label(self.settings.network_profile == NetworkProfile::P0"),
        "Settings must edit P0–P2"
    );
    let connect = ui
        .find("fn ui_connect_primary")
        .expect("ui_connect_primary");
    let primary = &ui[connect..ui.find("fn ui_network_advanced").unwrap_or(ui.len())];
    assert!(
        !primary.contains("apply_profile(NetworkProfile::P0)"),
        "System ui_connect_primary must not edit P0"
    );
}

#[test]
fn phase_x_rfc_0236_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0236-help-f1-model-path.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0236 missing after #353");
    for needle in [
        "#353",
        "Help F1 model path",
        "Settings",
        "Scan",
        "Compare",
        "first OPEN #354",
    ] {
        assert!(text.contains(needle), "RFC-0236 missing: {needle}");
    }
}

#[test]
fn phase_x_help_f1_model_path_runtime_present() {
    for rel in [
        "docs/help/en/model.select.md",
        "docs/help/uk/model.select.md",
        "docs/help/en/model.unavailable.md",
        "docs/help/uk/model.unavailable.md",
    ] {
        let text = std::fs::read_to_string(repo_root().join(rel)).unwrap();
        assert!(
            text.contains("Settings") || text.contains("Параметри"),
            "{rel}"
        );
        assert!(text.contains("Scan") || text.contains("Скан"), "{rel}");
        assert!(text.contains("offline") || text.contains("офлайн"), "{rel}");
        assert!(!text.to_lowercase().contains("must use cli"), "{rel}");
        // After #354, Compare is an available Work order (fail-closed; no silent substitute).
        if rel.contains("model.select") {
            assert!(
                text.contains("Compare") || text.contains("Порівняти"),
                "{rel} must document Compare"
            );
            assert!(
                text.to_lowercase().contains("silent")
                    || text.contains("тихої")
                    || text.contains("тихо"),
                "{rel} must mention no silent substitute"
            );
        }
    }
}

#[test]
fn phase_x_rfc_0237_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0237-work-compare-mode.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0237 missing after #354");
    for needle in [
        "#354",
        "Work Compare",
        "silent substitute",
        "RequireNewExecution",
        "first OPEN #355",
    ] {
        assert!(text.contains(needle), "RFC-0237 missing: {needle}");
    }
}

#[test]
fn phase_x_work_compare_runtime_present() {
    let wr = std::fs::read_to_string(
        repo_root().join("crates/aira-desktop-runtime/src/work_readiness.rs"),
    )
    .unwrap();
    assert!(wr.contains("WorkExecutorPreference::Compare") || wr.contains("Compare {"));
    assert!(wr.contains("RequireNewExecution"));
    assert!(wr.contains("admission_b"));
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(ui.contains("work_executor_compare") || ui.contains("WorkExecutorUiMode::Compare"));
    let jobs =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/async_jobs.rs")).unwrap();
    assert!(jobs.contains("try_spawn_compare_submit"));
}

#[test]
fn phase_x_rfc_0238_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0238-model-data-paths.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0238 missing after #355");
    for needle in [
        "#355",
        "Model data paths",
        "storage",
        "unknown",
        "first OPEN #356",
    ] {
        assert!(text.contains(needle), "RFC-0238 missing: {needle}");
    }
}

#[test]
fn phase_x_model_data_paths_runtime_present() {
    let ms = std::fs::read_to_string(
        repo_root().join("crates/aira-desktop-runtime/src/model_storage.rs"),
    )
    .unwrap();
    assert!(ms.contains("ModelStorageSnapshot"));
    assert!(ms.contains("load_model_storage"));
    assert!(ms.contains("available_bytes"));
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(ui.contains("settings_models_storage"));
    assert!(ui.contains("model_storage"));
}

#[test]
fn phase_x_rfc_0233_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0233-mock-honesty-result-triple.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0233 missing after #350");
    for needle in [
        "#350",
        "Mock honesty",
        "requested",
        "applied",
        "executed",
        "first OPEN #351",
    ] {
        assert!(text.contains(needle), "RFC-0233 missing: {needle}");
    }
}

#[test]
fn phase_x_mock_honesty_runtime_present() {
    let wv =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/work_view.rs")).unwrap();
    assert!(wv.contains("ResultModelTriple"));
    assert!(wv.contains("WorkSubmitModelContext"));
    assert!(wv.contains("EXECUTED_MOCK_LABEL"));
    assert!(wv.contains("#350") || wv.contains("RFC-0233"));
    let ui =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/ui.rs")).unwrap();
    assert!(ui.contains("work_mock_banner"));
    assert!(ui.contains("work_triple_requested"));
}

#[test]
fn phase_x_rfc_0232_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0232-work-executor-capability-readiness.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0232 missing after #349");
    for needle in [
        "#349",
        "Work executor",
        "math",
        "generate",
        "AdmissionConstraints",
        "first OPEN #351",
    ] {
        assert!(text.contains(needle), "RFC-0232 missing: {needle}");
    }
}

#[test]
fn phase_x_rfc_0231_present() {
    let path = repo_root().join("specs/rfc/AIRA-RFC-0231-settings-models-catalog-gui.md");
    let text = std::fs::read_to_string(&path).expect("RFC-0231 missing after #348");
    for needle in ["#348", "Settings", "catalog", "first OPEN #350", "RFC-0230"] {
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
fn phase_x_work_executor_readiness_runtime_present() {
    let wr = std::fs::read_to_string(
        repo_root().join("crates/aira-desktop-runtime/src/work_readiness.rs"),
    )
    .unwrap();
    assert!(wr.contains("evaluate_work_readiness"));
    assert!(wr.contains("WorkExecutorPreference"));
    assert!(wr.contains("WorkCapabilityKind"));
    assert!(wr.contains("#349") || wr.contains("RFC-0232"));
    let work =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/app/work.rs")).unwrap();
    assert!(work.contains("WorkModelUnready") || work.contains("refresh_work_readiness"));
    let actions =
        std::fs::read_to_string(repo_root().join("crates/aira-desktop/src/actions.rs")).unwrap();
    assert!(actions.contains("submit_problem_with_admission"));
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
        "#350",
        "#351",
        "#352",
        "#353",
        "#354",
        "#355",
        "#356",
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
    assert!(readme.contains("#356") || readme.contains("first OPEN"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-x-plan.md"));
    assert!(docs.contains("IN PROGRESS") || docs.contains("#356"));
}

#[test]
fn phase_x_next_problem() {
    let text = std::fs::read_to_string(repo_root().join("NEXT_PROBLEM.md")).unwrap();
    assert!(text.contains("phase-x-plan.md") || text.contains("Phase X"));
    assert!(text.contains("#356") || text.contains("перший OPEN"));
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
    assert!(status.contains("#356"));
    assert!(status.contains("RFC-0238") || status.contains("0238"));
    assert!(status.contains("RFC-0237") || status.contains("0237"));
    assert!(status.contains("RFC-0236") || status.contains("0236"));
    assert!(status.contains("RFC-0235") || status.contains("0235"));
    assert!(status.contains("RFC-0234") || status.contains("0234"));
    assert!(status.contains("RFC-0233") || status.contains("0233"));
    assert!(status.contains("RFC-0231") || status.contains("0231"));
}
