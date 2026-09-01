//! Phase J wiring contract (#199): plan + QUEUE + cross-links.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn phase_j_plan_present() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-j-plan.md")).unwrap();
    for needle in [
        "Phase J",
        "#199",
        "#200",
        "#208",
        "J0 govern + Book II ceiling honesty",
        "object_store_access",
        "B1-010",
        "Book II",
        "AIRA-RFC-0096",
        "confirmed free",
        "**IN PROGRESS**",
        "first OPEN `#208`",
        "GPU marketplace",
    ] {
        assert!(text.contains(needle), "phase-j-plan missing: {needle}");
    }
    assert!(
        !text.contains("не в QUEUE"),
        "phase-j-plan must not stay 'не в QUEUE' after wiring"
    );
}

#[test]
fn phase_j_queue_wiring_199_done() {
    let text = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(
        text.contains("phase-j-plan.md"),
        "QUEUE missing phase-j-plan"
    );
    assert!(
        text.contains("| 199 | **DONE**"),
        "QUEUE #199 must be DONE after wiring"
    );
    assert!(
        text.contains("| 200 | **DONE**"),
        "QUEUE #200 must be DONE after Book II ceiling honesty"
    );
    assert!(
        text.contains("| 201 | **DONE**"),
        "QUEUE #201 must be DONE after sealing object_store_access"
    );
    assert!(
        text.contains("| 202 | **DONE**"),
        "QUEUE #202 must be DONE after VRA runtime B1-010"
    );
    assert!(
        text.contains("| 203 | **DONE**"),
        "QUEUE #203 must be DONE after event-log authority"
    );
    assert!(
        text.contains("| 204 | **DONE**"),
        "QUEUE #204 must be DONE after Reduction catalog bind"
    );
    assert!(
        text.contains("| 205 | **DONE**"),
        "QUEUE #205 must be DONE after semantic verify text.*"
    );
    assert!(
        text.contains("| 206 | **DONE**"),
        "QUEUE #206 must be DONE after evidence primacy runtime"
    );
    assert!(
        text.contains("| 207 | **DONE**"),
        "QUEUE #207 must be DONE after epistemic emit on C1"
    );
    assert!(
        text.contains("| 208 | **OPEN**"),
        "QUEUE first OPEN must be #208"
    );
    assert!(
        !text.contains("| 199 | **OPEN**"),
        "QUEUE #199 must not stay OPEN after wiring"
    );
    assert!(
        !text.contains("| 200 | **OPEN**"),
        "QUEUE #200 must not stay OPEN after ceiling honesty"
    );
    assert!(
        !text.contains("| 201 | **OPEN**"),
        "QUEUE #201 must not stay OPEN after Handle seal"
    );
    assert!(
        !text.contains("| 202 | **OPEN**"),
        "QUEUE #202 must not stay OPEN after VRA runtime B1-010"
    );
    assert!(
        !text.contains("| 203 | **OPEN**"),
        "QUEUE #203 must not stay OPEN after event-log authority"
    );
    assert!(
        !text.contains("| 204 | **OPEN**"),
        "QUEUE #204 must not stay OPEN after Reduction catalog bind"
    );
    assert!(
        !text.contains("| 205 | **OPEN**"),
        "QUEUE #205 must not stay OPEN after semantic verify text.*"
    );
    assert!(
        !text.contains("| 206 | **OPEN**"),
        "QUEUE #206 must not stay OPEN after evidence primacy runtime"
    );
    assert!(
        !text.contains("| 207 | **OPEN**"),
        "QUEUE #207 must not stay OPEN after epistemic emit on C1"
    );
    assert!(
        !text.contains("| 208 | **DONE**"),
        "QUEUE #208 must not be DONE at #207"
    );
    for n in 208..=208 {
        let open = format!("| {n} | **OPEN**");
        assert!(text.contains(&open), "QUEUE missing {open}");
        let done = format!("| {n} | **DONE**");
        assert!(
            !text.contains(&done),
            "QUEUE {n} must not be DONE at wiring"
        );
    }
    for needle in [
        "J0 govern + Book II ceiling honesty",
        "J1 Book I remainder",
        "RFC-0096",
        "first OPEN `#208`",
        "Analyze-234",
        "Analyze-235",
        "Analyze-236",
        "Analyze-237",
        "Analyze-238",
        "Analyze-239",
        "Analyze-240",
        "Analyze-241",
        "Analyze-242",
        "Analyze-243",
    ] {
        assert!(text.contains(needle), "QUEUE missing: {needle}");
    }
}

#[test]
fn phase_j_readme_and_docs_index() {
    let readme = std::fs::read_to_string(repo_root().join("README.md")).unwrap();
    assert!(readme.contains("phase-j-plan.md"));
    assert!(readme.contains("#208"));
    assert!(readme.contains("#207"));
    assert!(readme.contains("#206"));
    assert!(readme.contains("#205"));
    assert!(readme.contains("#204"));
    assert!(readme.contains("#203"));
    assert!(readme.contains("#202"));
    assert!(readme.contains("#201"));
    assert!(readme.contains("#200"));
    assert!(readme.contains("#199"));
    let docs = std::fs::read_to_string(repo_root().join("docs/README.md")).unwrap();
    assert!(docs.contains("phase-j-plan.md"));
    assert!(docs.contains("#199"));
    assert!(docs.contains("first OPEN `#208`"));
    assert!(docs.contains("#208"));
    assert!(docs.contains("**IN PROGRESS**"));
}

#[test]
fn phase_i_points_to_active_phase_j() {
    let text = std::fs::read_to_string(repo_root().join("docs/phase-i-plan.md")).unwrap();
    assert!(text.contains("phase-j-plan.md"));
    assert!(text.contains("#199"));
    assert!(text.contains("first OPEN `#208`"));
}

#[test]
fn phase_j_rfc_0096_id_free() {
    let rfc_dir = repo_root().join("specs/rfc");
    for entry in std::fs::read_dir(&rfc_dir).expect("specs/rfc") {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            !name.starts_with("AIRA-RFC-0096"),
            "RFC-0096 reserved for J close; unexpected file: {name}"
        );
    }
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    assert!(status.contains("Phase J gates"));
    assert!(status.contains("RFC-0096"));
    assert!(status.contains("phase_j_doc.rs"));
    assert!(status.contains("#200"));
    assert!(status.contains("| #199 | Phase J wiring + contract"));
}

#[test]
fn phase_j_book_ii_ceiling_200() {
    let text = std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "## Book II — protocols",
        "local adapter = v0.3 ceiling",
        "Common envelope + signature",
        "Unsupported version without side effects",
        "Event Protocol publish idempotency",
        "Artifact Protocol publish/resolve + hash",
        "Identity descriptor",
        "Discovery by Capability, not Node",
        "Capability Advertisement",
        "CRP",
        "Settlement / Audit protocol",
        "R2 Local Protocol Node",
        "Object / Artifact / Event stores",
        "| #200 | Book II ceiling honesty",
        "**DONE** @ this PR",
        "not Book II wire mesh",
    ] {
        assert!(
            text.contains(needle),
            "implementation-status #200 ceiling missing: {needle}"
        );
    }
    assert!(
        text.contains("| Federation protocol (full) |"),
        "federation row must remain in Book II"
    );
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 200 | **DONE**"));
    assert!(queue.contains("| 201 | **DONE**"));
}

#[test]
fn phase_j_seal_object_store_access_201() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0097",
        "store-backend",
        "object_store_access_is_not_in_the_default_prelude",
        "store_backend_feature_is_only_enabled_by_aira_core",
        "csu_sources_do_not_import_object_store_access",
        "| #201 | Seal `object_store_access`",
        "**DONE** @ this PR",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #201 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0097-seal-object-store-access.md"),
    )
    .expect("RFC-0097");
    for needle in ["#201", "store-backend", "object_store_access", "not a CSU"] {
        assert!(rfc.contains(needle), "RFC-0097 missing: {needle}");
    }
    let cargo = std::fs::read_to_string(repo_root().join("crates/aira-object/Cargo.toml")).unwrap();
    assert!(cargo.contains("store-backend"));
    let core = std::fs::read_to_string(repo_root().join("crates/aira-core/Cargo.toml")).unwrap();
    assert!(core.contains("features = [\"store-backend\"]"));
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 201 | **DONE**"));
    assert!(queue.contains("| 202 | **DONE**"));
}

#[test]
fn phase_j_vra_runtime_202() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0098",
        "verified_result_body_has_b1_010_required_keys",
        "missing_vra_required",
        "| #202 | VRA runtime B1-010",
        "**DONE** @ this PR",
        "c1.pipeline.calculate_2_plus_2",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #202 missing: {needle}"
        );
    }
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0098-vra-runtime-b1-010.md"))
            .expect("RFC-0098");
    for needle in ["#202", "required[]", "calculate_2_plus_2", "B1-010"] {
        assert!(rfc.contains(needle), "RFC-0098 missing: {needle}");
    }
    let schema = std::fs::read_to_string(
        repo_root().join("schemas/result/verified-result-artifact.schema.json"),
    )
    .unwrap();
    for key in [
        "result_id",
        "problem_statement_ref",
        "context_ref",
        "solution_refs",
        "evidence_refs",
        "verification_status",
        "confidence",
        "scope",
        "provenance_refs",
        "artifact_hash",
        "signature",
        "created_at",
    ] {
        assert!(
            schema.contains(&format!("\"{key}\"")),
            "schema missing {key}"
        );
    }
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 202 | **DONE**"));
    assert!(queue.contains("| 203 | **DONE**"));
    let src =
        std::fs::read_to_string(repo_root().join("csu/verification-basic/src/lib.rs")).unwrap();
    for needle in [
        "seal_vra_body",
        "vra_binding_refs",
        "result_id",
        "problem_statement_ref",
        "context_ref",
        "solution_refs",
        "artifact_hash",
        "created_at",
    ] {
        assert!(src.contains(needle), "verification-basic missing {needle}");
    }
}

#[test]
fn phase_j_event_log_authority_203() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0099",
        "event_tail_after_reopen_reads_file_chain_not_memory_or_legacy",
        "| #203 | Event-log authority",
        "**DONE** @ this PR",
        "file-chain-log.json",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #203 missing: {needle}"
        );
    }
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0099-event-log-authority.md"))
            .expect("RFC-0099");
    for needle in ["#203", "file-chain-log.json", "event_tail", "drain_from"] {
        assert!(rfc.contains(needle), "RFC-0099 missing: {needle}");
    }
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 203 | **DONE**"));
    assert!(queue.contains("| 204 | **DONE**"));
    let src = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/local.rs")).unwrap();
    assert!(src.contains("FileChainEventLog::open(self.paths.file_chain_event_log())"));
    assert!(src.contains("not `OperationalPlane`"));
}

#[test]
fn phase_j_reduction_catalog_204() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0100",
        "plane_reduction_binds_reuse_index_without_enable_ready_solution",
        "open_with_reuse_index",
        "| #204 | Reduction catalog bind",
        "**DONE** @ this PR",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #204 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0100-reduction-catalog-bind.md"),
    )
    .expect("RFC-0100");
    for needle in [
        "#204",
        "reuse-index",
        "enable_ready_solution",
        "submit_problem",
    ] {
        assert!(rfc.contains(needle), "RFC-0100 missing: {needle}");
    }
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 204 | **DONE**"));
    assert!(queue.contains("| 205 | **DONE**"));
    assert!(queue.contains("| 206 | **DONE**"));
    let plane = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/plane.rs")).unwrap();
    assert!(plane.contains("open_with_reuse_index"));
    assert!(plane.contains("bind_catalog_for_text"));
    let test_src =
        std::fs::read_to_string(repo_root().join("crates/aira-flow/src/lib.rs")).unwrap();
    let ready_fn = test_src
        .split("fn ready_solution_reuse_skips_execution")
        .nth(1)
        .unwrap()
        .split("fn plane_reduction_binds")
        .next()
        .unwrap();
    assert!(
        !ready_fn.contains("enable_ready_solution"),
        "ready_solution_reuse_skips_execution must not call enable_ready_solution"
    );
}

#[test]
fn phase_j_semantic_verify_text_205() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0101",
        "wrong_text_echo_result_is_not_verified",
        "wrong_text_uppercase_result_is_not_verified",
        "| #205 | Semantic verify text.*",
        "**DONE** @ this PR",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #205 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0101-semantic-verify-text.md"),
    )
    .expect("RFC-0101");
    for needle in [
        "#205",
        "text.echo",
        "text.uppercase",
        "VerificationFailed",
        "to_uppercase",
    ] {
        assert!(rfc.contains(needle), "RFC-0101 missing: {needle}");
    }
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 205 | **DONE**"));
    assert!(queue.contains("| 206 | **DONE**"));
    assert!(queue.contains("| 207 | **DONE**"));
    let src =
        std::fs::read_to_string(repo_root().join("csu/verification-basic/src/lib.rs")).unwrap();
    assert!(src.contains("text_matches_claimed"));
    assert!(src.contains("action_expression"));
    assert!(
        !src.contains("body.get(\"result\").and_then(|v| v.as_str()).is_some()"),
        "text.* must not stay presence-only"
    );
}

#[test]
fn phase_j_evidence_primacy_206() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0102",
        "claim_without_evidence_rejected_as_operational_input",
        "assumption_without_evidence_is_operational_input",
        "| #206 | Evidence primacy runtime",
        "**DONE** @ this PR",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #206 missing: {needle}"
        );
    }
    let rfc = std::fs::read_to_string(
        repo_root().join("specs/rfc/AIRA-RFC-0102-evidence-primacy-runtime.md"),
    )
    .expect("RFC-0102");
    for needle in [
        "#206",
        "claim_kind",
        "EvidencePrimacy",
        "Assumption",
        "Hypothesis",
    ] {
        assert!(rfc.contains(needle), "RFC-0102 missing: {needle}");
    }
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 206 | **DONE**"));
    assert!(queue.contains("| 207 | **DONE**"));
    assert!(!queue.contains("| 207 | **OPEN**"));
    let src = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/plane.rs")).unwrap();
    assert!(src.contains("reject_claim_without_evidence"));
    assert!(src.contains("EvidencePrimacy"));
    let evi = std::fs::read_to_string(repo_root().join("csu/evidence-basic/src/lib.rs")).unwrap();
    assert!(evi.contains("claim_lacks_required_evidence"));
}

#[test]
fn phase_j_epistemic_emit_207() {
    let status =
        std::fs::read_to_string(repo_root().join("docs/implementation-status.md")).unwrap();
    for needle in [
        "RFC-0103",
        "calculate_two_plus_two_emits_epistemic_assessment",
        "| #207 | Epistemic emit on C1",
        "**DONE** @ this PR",
    ] {
        assert!(
            status.contains(needle),
            "implementation-status #207 missing: {needle}"
        );
    }
    let rfc =
        std::fs::read_to_string(repo_root().join("specs/rfc/AIRA-RFC-0103-epistemic-emit-c1.md"))
            .expect("RFC-0103");
    for needle in [
        "#207",
        "epistemic-assessment",
        "submit_problem",
        "full Epistemic plane",
    ] {
        assert!(rfc.contains(needle), "RFC-0103 missing: {needle}");
    }
    let queue = std::fs::read_to_string(repo_root().join("QUEUE.md")).unwrap();
    assert!(queue.contains("| 207 | **DONE**"));
    assert!(queue.contains("| 208 | **OPEN**"));
    assert!(!queue.contains("| 208 | **DONE**"));
    let plane = std::fs::read_to_string(repo_root().join("crates/aira-flow/src/plane.rs")).unwrap();
    assert!(plane.contains("pipeline produced no epistemic assessment"));
    let c1 =
        std::fs::read_to_string(repo_root().join("crates/aira-conformance/src/c1.rs")).unwrap();
    assert!(c1.contains("latest_epistemic_assessment"));
    assert!(c1.contains("aira:schema:epistemic:assessment:0.1"));
    assert!(c1.contains("EpistemicBasicCsu"));
}
