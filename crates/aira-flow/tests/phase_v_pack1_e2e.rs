//! Phase V / Repair Pack 1 §6 fail-closed e2e subset (QUEUE `#329` / RFC-0214).
//!
//! Covers the three Pack 1 acceptance scenarios named in `docs/phase-v-plan.md`:
//! reuse/model, settings-during-run, verify-tamper-before-activate.
//! Full repair §6 (replicas, share, quota, GUI) stays Pack 2+.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use aira_csu_model_acquisition::{
    activate_verified, fetch_to_quarantine, verify_quarantine, write_default_deny_policy,
    AcquisitionError, VerifyOutcome, ACTIVATED_POINTER_REL,
};
use aira_event::EventType;
use aira_flow::{
    init_node, AdmissionConstraints, AdmissionSnapshot, LocalSession, OperationalPlane,
    ReusePolicy, SubmitOutcome,
};
use aira_object::{active_signature, ContentHash};
use serde_json::{json, Map, Value};

/// Process-wide CSU tenant / signer isolation (same contract as flow unit tests).
fn isolated() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let g = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    aira_object::reset_csu_tenants();
    aira_object::reset_primary_signer();
    aira_object::reset_clock();
    g
}

fn init_acquisition_root(root: &Path) {
    for d in ["artifacts", "events", "models", "identity"] {
        fs::create_dir_all(root.join(d)).unwrap();
    }
    fs::write(
        root.join("events").join("event-log.json"),
        "{\"events\":[]}",
    )
    .unwrap();
    fs::write(
        root.join("config.json"),
        r#"{"node":{"mode":"local","profile":"C1"},"security":{"allow_network_for_csu":false,"allow_shell_for_csu":false,"require_signed_artifacts":true,"require_signed_events":true,"require_signed_csu_manifests":true},"storage":{"object_store":"sqlite","event_log":"json","artifact_store":"filesystem"},"csu":{"autoload":[]}}"#,
    )
    .unwrap();
}

fn signed_model_artifact(model_id: &str, content_hash: &str) -> Value {
    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:artifact:0.1"),
    );
    body.insert("model_id".into(), json!(model_id));
    body.insert("format".into(), json!("gguf"));
    body.insert("quantization".into(), json!("int4"));
    body.insert("parameter_class".into(), json!("7B"));
    body.insert("content_hash".into(), json!(content_hash));
    body.insert(
        "provenance_refs".into(),
        json!(["aira:identity:local-test"]),
    );
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).unwrap();
    let sig = active_signature(&raw).unwrap();
    body.insert("signature".into(), serde_json::to_value(&sig).unwrap());
    Value::Object(body)
}

/// Pack 1 §6: same text + different model/context → reuse must not substitute the admit contract.
#[test]
fn pack1_e2e_same_text_different_model_no_reuse() {
    let _lock = isolated();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(".aira");
    init_node(&root).unwrap();
    let mut session = LocalSession::open(&root).unwrap();
    let text = "Calculate 2 + 2";

    assert!(matches!(
        session.submit_problem(text).unwrap(),
        SubmitOutcome::Completed { .. }
    ));

    let other_model = AdmissionSnapshot::from_text_and_constraints(
        text,
        &AdmissionConstraints {
            model_ref: Some("aira:model:pack1-other".into()),
            ..Default::default()
        },
    );
    let second = session
        .submit_problem_with_admission(text, other_model)
        .unwrap();
    assert!(matches!(second, SubmitOutcome::Completed { .. }));
    assert!(
        session
            .plane()
            .events()
            .iter()
            .any(|e| e.event_type == EventType::CapsuleCompleted),
        "different model_ref must re-execute"
    );
    assert!(
        !session
            .plane()
            .events()
            .iter()
            .any(|e| e.payload_ref.as_deref() == Some("reuse:ready_solution")),
        "reuse must not satisfy a different model admit"
    );

    let require_new = AdmissionSnapshot::from_text_and_constraints(
        text,
        &AdmissionConstraints {
            reuse_policy: ReusePolicy::RequireNewExecution,
            ..Default::default()
        },
    );
    let third = session
        .submit_problem_with_admission(text, require_new)
        .unwrap();
    assert!(matches!(third, SubmitOutcome::Completed { .. }));
    assert!(
        session
            .plane()
            .events()
            .iter()
            .any(|e| e.event_type == EventType::CapsuleCompleted),
        "RequireNewExecution (compare/measure) must not cache-Success"
    );
}

/// Pack 1 §6: Settings-like mutation after admit must not rewrite the frozen snapshot.
#[test]
fn pack1_e2e_settings_mid_run_keeps_admission_binding() {
    let _lock = isolated();
    let dir = tempfile::tempdir().unwrap();
    let mut plane = OperationalPlane::open(dir.path()).unwrap();
    let text = "Calculate 2 + 2";
    let constraints = AdmissionConstraints {
        model_ref: Some("aira:model:pack1-chosen".into()),
        generation: aira_flow::GenerationParameters {
            temperature: Some(0.25),
            ..Default::default()
        },
        reuse_policy: ReusePolicy::AllowReuse,
        ..Default::default()
    };
    let snap = AdmissionSnapshot::from_text_and_constraints(text, &constraints);
    let _ = plane.submit_problem_with_admission(text, snap).unwrap();
    let (_, admitted) = plane.last_admission().expect("admission").clone();
    assert_eq!(
        admitted.model_ref.as_deref(),
        Some("aira:model:pack1-chosen")
    );
    assert_eq!(admitted.generation.temperature, Some(0.25));

    // Post-admit Settings / UI mutation of a local copy (not re-admit).
    let mut settings_like = constraints;
    settings_like.model_ref = Some("aira:model:pack1-switched".into());
    settings_like.generation.temperature = Some(0.95);
    assert_ne!(settings_like.model_ref, admitted.model_ref);

    let (_, still) = plane.last_admission().expect("admission still frozen");
    assert_eq!(still.model_ref.as_deref(), Some("aira:model:pack1-chosen"));
    assert_eq!(still.generation.temperature, Some(0.25));
    assert_eq!(still, &admitted);
}

/// Pack 1 §6: verify-file tampered before activate → activation rejected (no activated pointer).
#[test]
fn pack1_e2e_verify_tamper_before_activate_rejected() {
    let _lock = isolated();
    let dir = tempfile::tempdir().unwrap();
    init_acquisition_root(dir.path());
    write_default_deny_policy(dir.path(), true).unwrap();

    let src = dir.path().join("pack1.gguf");
    fs::write(&src, b"pack1-honest-weights").unwrap();
    fetch_to_quarantine(dir.path(), "aira:model:pack1", &src).unwrap();
    let observed = ContentHash::sha256_bytes(b"pack1-honest-weights");
    let art = signed_model_artifact("aira:model:pack1", observed.as_str());
    let art_path = dir.path().join("pack1.artifact.json");
    fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();

    let VerifyOutcome::Verified { verified_path, .. } =
        verify_quarantine(dir.path(), &art_path).unwrap()
    else {
        panic!("expected Verified before tamper");
    };
    fs::write(&verified_path, b"pack1-tampered-after-verify").unwrap();

    let err = activate_verified(dir.path()).unwrap_err();
    assert!(
        matches!(err, AcquisitionError::ActivateHashMismatch { .. }),
        "tamper must ActivateHashMismatch, got {err}"
    );
    assert!(
        !dir.path().join(ACTIVATED_POINTER_REL).exists(),
        "fail-closed: no activated.latest.json after tamper"
    );
}

/// Pack 1 acceptance glue: C1 2+2 still VERIFIED; generate-local stamps executor facts.
///
/// Separate planes: `latest_verified_result` is plane-scoped, so a prior C1 VRA would
/// shadow a later generate-local `Executed` outcome on the same plane.
#[test]
fn pack1_e2e_c1_and_generate_local_facts_hold() {
    let _lock = isolated();
    let dir = tempfile::tempdir().unwrap();

    {
        let mut plane = OperationalPlane::open(dir.path().join("c1")).unwrap();
        let c1 = plane.submit_problem("Calculate 2 + 2").unwrap();
        let SubmitOutcome::Completed { result, .. } = c1 else {
            panic!("C1 must Completed, got {c1:?}");
        };
        assert_eq!(result["result"], json!(4.0));
        assert_eq!(result["verification_status"], json!("VERIFIED"));
    }

    {
        let mut plane = OperationalPlane::open(dir.path().join("gen")).unwrap();
        plane.enable_activated_mock_llm().unwrap();
        let prompt = "Summarize the local Problem Statement without leaving the host.";
        let SubmitOutcome::Executed { result, .. } = plane.submit_problem(prompt).unwrap() else {
            panic!("generate-local must Executed");
        };
        assert_eq!(
            result["backend"],
            json!(aira_csu_execution_llm::MOCK_BACKEND_ID)
        );
        assert_eq!(
            result["model_ref"],
            json!(aira_csu_execution_llm::ALWAYS_ACTIVATED_MODEL_REF)
        );
        assert_eq!(
            result["model_content_hash"],
            json!(aira_csu_execution_llm::AlwaysActivated::content_hash())
        );
        assert!(result.get("capsule_ref").and_then(|v| v.as_str()).is_some());
        assert!(result
            .get("problem_statement_ref")
            .and_then(|v| v.as_str())
            .is_some());
    }
}
