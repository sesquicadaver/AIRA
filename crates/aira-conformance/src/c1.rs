//! C1 conformance suite (Issues #65, #70; remapped RFC-0242).
//!
//! Pipeline cases drive [`aira_flow::OperationalPlane`] as the **C1 reference/demo**
//! plane, not as a production event/scheduler/federation runtime
//! (`docs/operational-plane.md`).
//!
//! **RFC-0242:** OP-001 / `Calculate 2 + 2` → VERIFIED 4.0 is **legacy non-normative**.
//! Normative C1 pipeline smoke is a real **process** executor (`Executed` ≠ VERIFIED).
//! VRA completeness is checked via schema fixtures, not math acceptance.

use std::fs;
use std::path::Path;

use aira_csu::support::make_event;
use aira_csu::{Csu, CsuManifest, CsuRegistry};
use aira_csu_artifact_basic::ArtifactBasicCsu;
use aira_csu_context_basic::ContextBasicCsu;
use aira_csu_epistemic_basic::EpistemicBasicCsu;
use aira_csu_evidence_basic::EvidenceBasicCsu;
use aira_csu_execution_basic::ExecutionBasicCsu;
use aira_csu_execution_llm::{AlwaysActivated, ProcessBackend};
use aira_csu_reduction_basic::ReductionBasicCsu;
use aira_csu_verification_basic::VerificationBasicCsu;
use aira_event::EventType;
use aira_flow::{OperationalPlane, SubmitOutcome};
use aira_object::AiraRef;
use aira_schema::SchemaRegistry;
use serde_json::Value;

use crate::report::ConformanceProfile;
use crate::runner::{fail, finalize_suite, pass, CaseResult, ConformanceError, SuiteResult};

/// Run the C1 conformance suite and emit a Conformance Report Artifact.
pub fn run_c1(artifact_root: impl AsRef<Path>) -> Result<SuiteResult, ConformanceError> {
    let cases = vec![
        test_process_executor_executed(artifact_root.as_ref()),
        test_csu_manifests(),
        test_csu_external_partner_fixture(artifact_root.as_ref()),
        test_verified_result_completeness_fixture(),
        test_verified_result_extended_fields(),
        test_failure_to_evidence(artifact_root.as_ref()),
    ];
    finalize_suite(ConformanceProfile::C1, cases, artifact_root)
}

fn load_registry() -> Result<SchemaRegistry, ConformanceError> {
    let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR"))
        .map_err(|e| ConformanceError::Schema(e.to_string()))?;
    SchemaRegistry::load(root.join("schemas")).map_err(|e| ConformanceError::Schema(e.to_string()))
}

/// B1-010: every `required[]` key from the VRA schema must be present on a body.
fn missing_vra_required(result: &Value) -> Result<(), String> {
    let root =
        aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).map_err(|e| e.to_string())?;
    let schema_text =
        fs::read_to_string(root.join("schemas/result/verified-result-artifact.schema.json"))
            .map_err(|e| e.to_string())?;
    let schema: Value = serde_json::from_str(&schema_text).map_err(|e| e.to_string())?;
    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "VRA schema missing required[]".to_string())?;
    for key in required {
        let Some(name) = key.as_str() else {
            continue;
        };
        if result.get(name).is_none() {
            return Err(format!("runtime VRA missing required {name}"));
        }
    }
    Ok(())
}

/// Normative C1 pipeline: host process executor → Executed (not VERIFIED).
///
/// Uses `/bin/echo` as a real process algorithm (not MockBackend, not math.eval.safe).
fn test_process_executor_executed(artifact_root: &Path) -> CaseResult {
    let id = "c1.pipeline.process_executor_executed";
    let dir = artifact_root.join("c1-pipeline");
    let mut plane = match OperationalPlane::open(&dir) {
        Ok(p) => p,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = plane.bind_process_backend(ProcessBackend::new("echo"), AlwaysActivated) {
        return fail(id, e.to_string());
    }
    let prompt = "Summarize the local Problem Statement without leaving the host.";
    match plane.submit_problem(prompt) {
        Ok(SubmitOutcome::Executed { result, .. }) => {
            if result.get("verification_status") == Some(&serde_json::json!("VERIFIED")) {
                return fail(id, "process smoke must not mint VERIFIED");
            }
            if result.get("backend") != Some(&serde_json::json!("process")) {
                return fail(id, format!("expected backend=process, got {result}"));
            }
        }
        Ok(SubmitOutcome::Completed { .. }) => {
            return fail(id, "expected Executed (not Completed/VERIFIED)");
        }
        Ok(other) => return fail(id, format!("expected Executed, got {other:?}")),
        Err(e) => return fail(id, e.to_string()),
    }
    if plane.has_verified_result_artifact() {
        return fail(
            id,
            "process smoke must not create a Verified Result Artifact",
        );
    }
    pass(id)
}

/// Validate basic CSU manifests against schema (execution-basic kept as LEGACY CSU).
fn test_csu_manifests() -> CaseResult {
    let id = "c1.csu.manifests";
    let reg = match load_registry() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let manifests: Vec<CsuManifest> = vec![
        ContextBasicCsu::new().manifest().clone(),
        ReductionBasicCsu::new().manifest().clone(),
        ExecutionBasicCsu::new().manifest().clone(),
        VerificationBasicCsu::new().manifest().clone(),
        EvidenceBasicCsu::new().manifest().clone(),
        EpistemicBasicCsu::new().manifest().clone(),
        ArtifactBasicCsu::new().manifest().clone(),
    ];
    for m in manifests {
        if let Err(e) = m.validate_for_registration() {
            return fail(id, format!("{}: {e}", m.csu_id));
        }
        let v = match serde_json::to_value(&m) {
            Ok(v) => v,
            Err(e) => return fail(id, e.to_string()),
        };
        if let Err(e) = reg.validate("aira:schema:csu:manifest:0.1", &v) {
            return fail(id, format!("{} schema: {e}", m.csu_id));
        }
    }
    pass(id)
}

/// QUEUE #145 — third-party partner fixture loads into local registry (not a `csu/` crate).
fn test_csu_external_partner_fixture(artifact_root: &Path) -> CaseResult {
    let id = "c1.csu.external_partner_fixture";
    let root = match aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")) {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let path = root.join("fixtures/valid/csu/manifest-external-partner.json");
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return fail(id, e.to_string()),
    };
    let m: CsuManifest = match serde_json::from_str(&text) {
        Ok(m) => m,
        Err(e) => return fail(id, format!("parse: {e}")),
    };
    if m.csu_id.as_str() != "aira:csu:partner.external" {
        return fail(
            id,
            format!("expected aira:csu:partner.external, got {}", m.csu_id),
        );
    }
    if let Err(e) = m.validate_for_registration() {
        return fail(id, format!("validate_for_registration: {e}"));
    }
    let reg = match load_registry() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let v = match serde_json::to_value(&m) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = reg.validate("aira:schema:csu:manifest:0.1", &v) {
        return fail(id, format!("schema: {e}"));
    }
    let mut registry = CsuRegistry::new();
    if let Err(e) = registry.register(m.clone(), None) {
        return fail(id, format!("register: {e}"));
    }
    let reg_path = artifact_root
        .join("c1-external-partner")
        .join("registry.json");
    if let Some(parent) = reg_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return fail(id, e.to_string());
        }
    }
    if let Err(e) = registry.save(&reg_path) {
        return fail(id, format!("save: {e}"));
    }
    let loaded = match CsuRegistry::load(&reg_path) {
        Ok(r) => r,
        Err(e) => return fail(id, format!("load: {e}")),
    };
    if loaded.list().len() != 1 {
        return fail(id, format!("expected 1 entry, got {}", loaded.list().len()));
    }
    if loaded.list()[0].manifest.csu_id != m.csu_id {
        return fail(id, "loaded csu_id mismatch");
    }
    pass(id)
}

/// Verified Result Artifact completeness from fixture (not math 2+2 path).
fn test_verified_result_completeness_fixture() -> CaseResult {
    let id = "c1.result.verified_completeness";
    let root = match aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")) {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let reg = match load_registry() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let path = root.join("fixtures/valid/result/verified-result-artifact.json");
    if let Err(e) = reg.validate_file("aira:schema:result:verified-result-artifact:0.1", &path) {
        return fail(id, e.to_string());
    }
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return fail(id, e.to_string()),
    };
    let result: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    for key in [
        "verification_status",
        "confidence",
        "evidence_refs",
        "provenance_refs",
        "scope",
        "result_id",
        "solution_refs",
    ] {
        if result.get(key).is_none() {
            return fail(id, format!("missing field {key}"));
        }
    }
    if let Err(e) = missing_vra_required(&result) {
        return fail(id, e);
    }
    pass(id)
}

/// Extended VRA schema fields + B1-010 required coverage (QUEUE #126).
fn test_verified_result_extended_fields() -> CaseResult {
    let id = "c1.result.extended_fields";
    let reg = match load_registry() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let root = match aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")) {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let schema_id = "aira:schema:result:verified-result-artifact:0.1";
    let extended = root.join("fixtures/valid/result/verified-result-extended.json");
    if let Err(e) = reg.validate_file(schema_id, &extended) {
        return fail(id, format!("extended fixture: {e}"));
    }
    let text = match fs::read_to_string(&extended) {
        Ok(t) => t,
        Err(e) => return fail(id, e.to_string()),
    };
    let value: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    for key in [
        "counter_evidence_refs",
        "claim_refs",
        "revision_refs",
        "epistemic_status",
        "contextual_fitness",
        "source_output_ref",
    ] {
        if value.get(key).is_none() {
            return fail(id, format!("extended fixture missing {key}"));
        }
    }
    let schema_path = root.join("schemas/result/verified-result-artifact.schema.json");
    let schema_text = match fs::read_to_string(&schema_path) {
        Ok(t) => t,
        Err(e) => return fail(id, e.to_string()),
    };
    let schema: Value = match serde_json::from_str(&schema_text) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    let required = schema
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();
    for key in [
        "problem_statement_ref",
        "context_ref",
        "evidence_refs",
        "verification_status",
        "confidence",
        "scope",
        "provenance_refs",
        "artifact_hash",
        "signature",
        "result_id",
        "solution_refs",
        "created_at",
    ] {
        if !required.contains(&key) {
            return fail(id, format!("schema required missing B1-010 field {key}"));
        }
    }
    pass(id)
}

/// #70 — capsule failure → evidence; no fake verified result.
fn test_failure_to_evidence(artifact_root: &Path) -> CaseResult {
    let id = "c1.failure.to_evidence";
    let dir = artifact_root.join("c1-fail");
    let mut plane = match OperationalPlane::open(&dir) {
        Ok(p) => p,
        Err(e) => return fail(id, e.to_string()),
    };
    let missing = AiraRef::parse("aira:artifact:missing_conf_70").unwrap();
    let ev = make_event(
        "aira:event:conf_fail1",
        EventType::CapsuleCreated,
        vec![AiraRef::parse("aira:problem:conf_fail1").unwrap()],
        vec![missing],
        vec![],
        Some("text.generate.local".into()),
    );
    if let Err(e) = plane.inject_and_drain(ev) {
        return fail(id, e.to_string());
    }
    let events = plane.events();
    if !events
        .iter()
        .any(|e| e.event_type == EventType::CapsuleFailed)
    {
        return fail(id, "missing CapsuleFailed");
    }
    if !events
        .iter()
        .any(|e| e.event_type == EventType::FailureEvidenceCreated)
    {
        return fail(id, "missing FailureEvidenceCreated");
    }
    if plane.has_verified_result_artifact() {
        return fail(id, "fake Verified Result Artifact created");
    }
    if events
        .iter()
        .any(|e| e.event_type == EventType::VerificationCompleted)
    {
        return fail(id, "unexpected VerificationCompleted after failure");
    }
    pass(id)
}
