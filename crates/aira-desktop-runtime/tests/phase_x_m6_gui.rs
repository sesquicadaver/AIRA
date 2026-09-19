//! Phase X / Pack 2 M6 GUI facade acceptance (QUEUE `#357` / RFC-0240).
//!
//! Settings→Models prepare/select + Work readiness for two fixture models.
//! Not installed-product acceptance (fixture weights only).
//! CLI/HTTP admission e2e lives in `aira-flow` `phase_x_m6_acceptance`.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use aira_csu_model_acquisition::{
    activate_verified_model, fetch_to_quarantine, verify_quarantine, write_default_deny_policy,
    VerifyOutcome, ACTIVATED_POINTER_REL,
};
use aira_desktop_runtime::{
    evaluate_work_readiness, prepare_model, select_catalog_model, write_settings, CatalogSelection,
    DesktopPaths, DesktopSettings, LlmBackend, WorkExecutorPreference,
};
use aira_flow::init_node;
use aira_object::{active_signature, ContentHash};
use serde_json::{json, Map, Value};

fn isolated() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
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

fn init_m6_root(root: &Path) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    aira_object::reset_primary_signer();
    init_node(root).unwrap();
    write_default_deny_policy(root, true).unwrap();
    let mut rng = OsRng;
    let signing = SigningKey::generate(&mut rng);
    let id = format!("aira:identity:m6-gui.{}", uuid::Uuid::now_v7().as_simple());
    aira_object::create_or_ensure_node_identity(
        root,
        &id,
        "m6-gui",
        signing,
        aira_object::NodeIdentityCreatePolicy::Ensure,
    )
    .unwrap();
    aira_object::register_node_identity(root).unwrap();
}

fn quarantine_verify(root: &Path, name: &str, bytes: &[u8]) -> String {
    let src = root.join(format!("{name}.gguf"));
    fs::write(&src, bytes).unwrap();
    let model_ref = format!("aira:model:{name}");
    fetch_to_quarantine(root, &model_ref, &src).unwrap();
    let observed = ContentHash::sha256_bytes(bytes);
    let art = signed_model_artifact(&model_ref, observed.as_str());
    let art_path = root.join(format!("{name}.artifact.json"));
    fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
    match verify_quarantine(root, &art_path).unwrap() {
        VerifyOutcome::Verified { model_ref, .. } => model_ref,
        other => panic!("expected Verified, got {other:?}"),
    }
}

#[test]
fn m6_e2e_gui_catalog_and_work_readiness_two_models() {
    let _g = isolated();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(".aira");
    init_m6_root(&root);
    // RFC-0242: Work readiness requires host process LLM in settings.
    let paths = DesktopPaths::for_data_root(&root);
    let mut settings = DesktopSettings::default_p0(&paths);
    settings.llm_backend = LlmBackend::Process;
    settings.llm_ollama_model = Some("llama3:latest".into());
    write_settings(&paths, &settings).unwrap();

    let model_a = quarantine_verify(&root, "m6-gui-a", b"gui-a-weights");
    let model_b = quarantine_verify(&root, "m6-gui-b", b"gui-b-weights-xx");

    // Settings→Models Prepare (activate) for both.
    prepare_model(&root, &model_a).unwrap();
    prepare_model(&root, &model_b).unwrap();

    let (chosen_a, snap_a) =
        select_catalog_model(&root, CatalogSelection::Required(model_a.clone())).unwrap();
    assert_eq!(chosen_a, model_a);
    assert_eq!(snap_a.tip_model_ref.as_deref(), Some(model_a.as_str()));

    let (chosen_b, snap_b) =
        select_catalog_model(&root, CatalogSelection::Required(model_b.clone())).unwrap();
    assert_eq!(chosen_b, model_b);
    assert_eq!(snap_b.tip_model_ref.as_deref(), Some(model_b.as_str()));

    let tip: Value =
        serde_json::from_str(&fs::read_to_string(root.join(ACTIVATED_POINTER_REL)).unwrap())
            .unwrap();
    assert_eq!(tip["model_ref"], json!(&model_b));

    let prompt = "Summarize the local Problem Statement without leaving the host.";
    let ready_a = evaluate_work_readiness(
        &root,
        &settings,
        prompt,
        WorkExecutorPreference::Required(model_a.clone()),
    );
    assert!(
        !ready_a.ready,
        "file weights are not Work-ready on Ollama: {:?}",
        ready_a.reasons
    );
    assert!(ready_a.reasons.iter().any(|s| s.contains("cannot run")));
    assert_eq!(
        ready_a.resolved_model_ref.as_deref(),
        Some(model_a.as_str())
    );

    let ready_b = evaluate_work_readiness(
        &root,
        &settings,
        prompt,
        WorkExecutorPreference::Required(model_b.clone()),
    );
    assert!(
        !ready_b.ready,
        "Required B must not be ready: {:?}",
        ready_b.reasons
    );

    let compare = evaluate_work_readiness(
        &root,
        &settings,
        prompt,
        WorkExecutorPreference::Compare {
            a: model_a.clone(),
            b: model_b.clone(),
        },
    );
    assert!(
        !compare.ready,
        "Compare of file weights must not be ready: {:?}",
        compare.reasons
    );
    assert_eq!(
        compare.resolved_model_ref.as_deref(),
        Some(model_a.as_str())
    );
    assert_eq!(
        compare.resolved_model_ref_b.as_deref(),
        Some(model_b.as_str())
    );

    // Fail-closed: Compare with missing leg does not substitute.
    let bad = evaluate_work_readiness(
        &root,
        &settings,
        prompt,
        WorkExecutorPreference::Compare {
            a: model_a,
            b: "aira:model:m6-missing".into(),
        },
    );
    assert!(!bad.ready, "Compare with missing leg must not be ready");
    assert!(
        bad.reasons.iter().any(|r| r.contains("fail-closed")
            || r.contains("not available")
            || r.contains("missing")
            || r.contains("unready")
            || r.contains("removed")
            || r.contains("no silent")),
        "must explain fail-closed, got {:?}",
        bad.reasons
    );

    // Tip select still works after Compare readiness.
    let _ = activate_verified_model(&root, &model_b);
}
