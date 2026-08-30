//! Phase H wiring contract (#152): plan + QUEUE + cross-links.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_h_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-h-plan.md")).unwrap();
    for needle in [
        "Phase H",
        "#152",
        "#183",
        "H1 durable stores",
        "H3 CRP",
        "H4 settlement",
        "H5 research promotion",
        "AIRA-RFC-0077",
        "без вилок",
    ] {
        assert!(text.contains(needle), "phase-h-plan missing: {needle}");
    }
}

#[test]
fn phase_h_queue_wiring_152_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-h-plan.md"),
        "QUEUE missing phase-h-plan"
    );
    assert!(
        text.contains("| 152 | **DONE**"),
        "QUEUE #152 must be DONE after wiring"
    );
    assert!(
        text.contains("| 153 | **DONE**"),
        "QUEUE #153 must be DONE after C3 governance"
    );
    assert!(
        text.contains("| 154 | **DONE**"),
        "QUEUE #154 must be DONE after hash-chain tip"
    );
    assert!(
        text.contains("| 155 | **DONE**"),
        "QUEUE #155 must be DONE after prefix recovery"
    );
    assert!(
        text.contains("| 156 | **DONE**"),
        "QUEUE #156 must be DONE after durable backend"
    );
    assert!(
        text.contains("| 157 | **DONE**"),
        "QUEUE #157 must be DONE after session durable wire"
    );
    assert!(
        text.contains("| 158 | **DONE**"),
        "QUEUE #158 must be DONE after Sqlite object path"
    );
    assert!(
        text.contains("| 159 | **DONE**"),
        "QUEUE #159 must be DONE after stores status rollup"
    );
    assert!(
        text.contains("| 160 | **DONE**"),
        "QUEUE #160 must be DONE after capability ad persist"
    );
    assert!(
        text.contains("| 161 | **DONE**"),
        "QUEUE #161 must be DONE after C3 capability case"
    );
    assert!(
        text.contains("| 162 | **DONE**"),
        "QUEUE #162 must be DONE after federation export deny"
    );
    assert!(
        text.contains("| 163 | **DONE**"),
        "QUEUE #163 must be DONE after C3 cases ≥6"
    );
    assert!(
        text.contains("| 164 | **DONE**"),
        "QUEUE #164 must be DONE after optional C3 CI"
    );
    assert!(
        text.contains("| 165 | **DONE**"),
        "QUEUE #165 must be DONE after CRP schema fixtures"
    );
    assert!(
        text.contains("| 166 | **DONE**"),
        "QUEUE #166 must be DONE after CRP local adapter"
    );
    assert!(
        text.contains("| 167 | **DONE**"),
        "QUEUE #167 must be DONE after CRP reject node route"
    );
    assert!(
        text.contains("| 168 | **DONE**"),
        "QUEUE #168 must be DONE after CRP multi-candidate gate"
    );
    assert!(
        text.contains("| 169 | **DONE**"),
        "QUEUE #169 must be DONE after CRP route events"
    );
    assert!(
        text.contains("| 170 | **DONE**"),
        "QUEUE #170 must be DONE after B2-006 C3 case"
    );
    assert!(
        text.contains("| 171 | **DONE**"),
        "QUEUE #171 must be DONE after CRP status PARTIAL"
    );
    assert!(
        text.contains("| 172 | **DONE**"),
        "QUEUE #172 must be DONE after settlement fixtures"
    );
    assert!(
        text.contains("| 173 | **DONE**"),
        "QUEUE #173 must be DONE after settlement receipt store"
    );
    assert!(
        text.contains("| 174 | **DONE**"),
        "QUEUE #174 must be DONE after B2-011 privacy smoke"
    );
    assert!(
        text.contains("| 175 | **DONE**"),
        "QUEUE #175 must be DONE after run_c4 scaffold"
    );
    assert!(
        text.contains("| 176 | **DONE**"),
        "QUEUE #176 must be DONE after settlement status PARTIAL"
    );
    assert!(
        text.contains("| 177 | **DONE**"),
        "QUEUE #177 must be DONE after RFC-P promotion doc"
    );
    assert!(
        text.contains("| 178 | **OPEN**"),
        "QUEUE #178 must be next OPEN"
    );
    assert!(text.contains("| 183 | **OPEN**"), "QUEUE missing #183");
    for needle in ["H0 govern", "H1 durable stores", "H3 CRP local", "RFC-0077"] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_h_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-h-plan.md"));
    assert!(readme.contains("#152"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-h-plan.md"));
    assert!(docs.contains("#152"));
}

#[test]
fn phase_g_points_to_phase_h() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-g-plan.md")).unwrap();
    assert!(text.contains("phase-h-plan.md"));
    assert!(text.contains("#152"));
}

#[test]
fn phase_h_rfc_p_promotion_doc() {
    let text = std::fs::read_to_string(repo_root().join("docs/rfc-p-promotion.md")).unwrap();
    for needle in [
        "RFC-P",
        "Book V → operational лише через promotion",
        "Direct Operational Authority",
        "Artifact Promotion Candidate",
        "non-operational until promote",
        "research or promotion-candidate artifact presented as operational input is **rejected**",
        "RFC-P **MUST** contain",
        "security analysis",
        "failure model",
        "replication evidence",
        "disabled by default",
        "AIRA-RES-PHM",
        "AIRA-RES-HIE",
        "AIRA-RES-GC",
        "#178",
        "#179",
        "#180",
        "run_c5",
        "QUEUE",
        "#177",
    ] {
        assert!(
            text.contains(needle),
            "rfc-p-promotion.md missing: {needle}"
        );
    }
    let index = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(
        index.contains("rfc-p-promotion.md"),
        "docs/README.md must index rfc-p-promotion.md"
    );
}

#[test]
fn phase_h_h1_stores_status_rollup() {
    let text = std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "Phase H gates",
        "H1 DONE",
        "#154",
        "#159",
        "FileChainEventLog",
        "plane_memory_beside_node_sqlite_object_path",
        "CapabilityAdvertisementStore",
        "capability_ad_persist_roundtrip",
        "c3.capability.advertisement",
        "federation_export_import_deny_by_default_audits",
        "c3.federation.export_deny",
        "conformance-c3",
        "aira:schema:protocol:crp-route-request:0.1",
        "aira:schema:protocol:crp-route-candidate:0.1",
        "LocalCrpAdapter",
        "RFC-0079",
        "crp_local_adapter_routes_capability_not_node",
        "c3.crp.reject_node_route",
        "crp_multi_candidate_and_policy_gate_bind",
        "crp.bind",
        "crp_route_events_selected_rejected_failure",
        "RouteSelected",
        "c3.crp.route_candidate",
        "Local in-process only",
        "aira:schema:settlement:receipt:0.1",
        "privacy_class",
        "settlement_receipt_schema_loads",
        "SettlementReceiptStore",
        "settlement_receipt_store_append_roundtrip_and_verify_on_read",
        "RFC-0080",
        "aira:settlement:receipts-jsonl:v1",
        "b2_011_settlement_privacy_smoke",
        "validate_settlement_privacy",
        "SETTLEMENT_PRIVACY_FORBIDDEN_KEYS",
        "c4.settlement.receipt_emit_verify",
        "run_c4",
        "RFC-0081",
        "Local audit receipts only",
        "no blockchain ledger",
        "rfc-p-promotion.md",
        "RFC-P promotion doc",
        "Book V → operational лише через promotion",
        "process only",
        "no runtime",
    ] {
        assert!(
            text.contains(needle),
            "implementation-status missing: {needle}"
        );
    }
}
