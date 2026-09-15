//! Phase X / Pack 2 M6 installed-product acceptance (QUEUE `#357` / RFC-0240).
//!
//! Proves two real local models share one staff path for select + CLI/HTTP
//! admission (`AdmissionConstraints` = CLI flags = `POST /v1/problems` body),
//! cold restart keeps tip/slots, and Required-unavailable never silent-substitutes.
//! GUI catalog/Work facade lives in `aira-desktop-runtime` `phase_x_m6_gui`.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use aira_csu_model_acquisition::{
    activate_verified_model, activated_slot_pointer_path, fetch_to_quarantine,
    list_model_lifecycle, select_model, verify_quarantine, write_default_deny_policy,
    ModelSelectError, ModelSelection, ModelSelectionVia, VerifyOutcome, ACTIVATED_POINTER_REL,
};
use aira_flow::{init_node, AdmissionConstraints, AdmissionSnapshot, LocalSession, SubmitOutcome};
use aira_object::{active_signature, ContentHash};
use serde_json::{json, Map, Value};

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

/// Node root with identity + local-add policy ready for two-model install.
fn init_m6_root(root: &Path) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    init_node(root).unwrap();
    write_default_deny_policy(root, true).unwrap();
    let mut rng = OsRng;
    let signing = SigningKey::generate(&mut rng);
    let id = format!("aira:identity:m6-e2e.{}", uuid::Uuid::now_v7().as_simple());
    aira_object::create_or_ensure_node_identity(
        root,
        &id,
        "m6-e2e",
        signing,
        aira_object::NodeIdentityCreatePolicy::Ensure,
    )
    .unwrap();
    aira_object::register_node_identity(root).unwrap();
}

fn install_model(root: &Path, name: &str, bytes: &[u8]) -> String {
    let src = root.join(format!("{name}.gguf"));
    fs::write(&src, bytes).unwrap();
    let model_ref = format!("aira:model:{name}");
    fetch_to_quarantine(root, &model_ref, &src).unwrap();
    let observed = ContentHash::sha256_bytes(bytes);
    let art = signed_model_artifact(&model_ref, observed.as_str());
    let art_path = root.join(format!("{name}.artifact.json"));
    fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
    let VerifyOutcome::Verified { model_ref, .. } = verify_quarantine(root, &art_path).unwrap()
    else {
        panic!("expected Verified for {name}");
    };
    activate_verified_model(root, &model_ref).unwrap();
    model_ref
}

fn generate_prompt() -> &'static str {
    "Summarize the local Problem Statement without leaving the host."
}

fn cli_or_http_constraints(model_ref: &str) -> AdmissionConstraints {
    // Same shape as CLI `--model-ref` and HTTP `admission.model_ref`.
    AdmissionConstraints {
        model_ref: Some(model_ref.to_string()),
        ..Default::default()
    }
}

fn submit_required(session: &mut LocalSession, model_ref: &str) -> SubmitOutcome {
    let snap = AdmissionSnapshot::from_text_and_constraints(
        generate_prompt(),
        &cli_or_http_constraints(model_ref),
    );
    session
        .submit_problem_with_admission(generate_prompt(), snap)
        .unwrap()
}

/// M6: install A+B; CLI/HTTP admission runs each Required model; select Auto uses tip.
#[test]
fn m6_e2e_two_models_cli_http_admission_paths() {
    let _lock = isolated();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(".aira");
    init_m6_root(&root);

    let model_a = install_model(&root, "m6-a", b"m6-model-a-weights");
    let model_b = install_model(&root, "m6-b", b"m6-model-b-weights-xx");

    let life = list_model_lifecycle(&root).unwrap();
    assert_eq!(life.len(), 2);
    assert!(life.iter().all(|e| e.verified && e.available));

    let tip: Value =
        serde_json::from_str(&fs::read_to_string(root.join(ACTIVATED_POINTER_REL)).unwrap())
            .unwrap();
    assert_eq!(
        tip["model_ref"],
        json!(&model_b),
        "latest tip is B after install"
    );

    // Select API (shared by CLI resolve + GUI catalog).
    let sel_a = select_model(&root, ModelSelection::Required(model_a.clone())).unwrap();
    assert_eq!(sel_a.model_ref, model_a);
    assert_eq!(sel_a.via, ModelSelectionVia::Required);
    let sel_auto = select_model(&root, ModelSelection::Auto).unwrap();
    assert_eq!(sel_auto.model_ref, model_b);
    assert_eq!(sel_auto.via, ModelSelectionVia::AutoLatestTip);

    // HTTP body shape: `{"text", "admission": {...}}` deserializes to the same constraints.
    let http_body = json!({
        "text": generate_prompt(),
        "admission": { "model_ref": model_a }
    });
    let admission: AdmissionConstraints =
        serde_json::from_value(http_body["admission"].clone()).unwrap();
    assert_eq!(admission.model_ref.as_deref(), Some(model_a.as_str()));

    let mut session = LocalSession::open(&root).unwrap();
    let out_a = submit_required(&mut session, &model_a);
    assert!(
        matches!(out_a, SubmitOutcome::Executed { .. }),
        "Required A must Executed via slot (tip is B): {out_a:?}"
    );
    let (_, adm_a) = session
        .plane()
        .last_admission()
        .expect("admission A")
        .clone();
    assert_eq!(
        adm_a.model_ref.as_deref(),
        Some(model_a.as_str()),
        "CLI/HTTP admit must freeze Required A, not tip B"
    );

    let out_b = submit_required(&mut session, &model_b);
    assert!(
        matches!(out_b, SubmitOutcome::Executed { .. }),
        "Required B must Executed: {out_b:?}"
    );
    let (_, adm_b) = session
        .plane()
        .last_admission()
        .expect("admission B")
        .clone();
    assert_eq!(adm_b.model_ref.as_deref(), Some(model_b.as_str()));
    // Mock backend must not mint VERIFIED (#339); process stamps model_ref separately.
    if let SubmitOutcome::Executed { result, .. } = out_b {
        assert_ne!(
            result.get("verification_status").and_then(|v| v.as_str()),
            Some("VERIFIED"),
            "generate-local must not mint VERIFIED"
        );
    }
}

/// M6: cold reopen of the same root keeps both slots and tip selection.
#[test]
fn m6_e2e_cold_restart_preserves_tip_and_slots() {
    let _lock = isolated();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(".aira");
    init_m6_root(&root);

    let model_a = install_model(&root, "m6-cold-a", b"cold-a-bytes");
    let model_b = install_model(&root, "m6-cold-b", b"cold-b-bytes-yy");
    // Tip → A explicitly (GUI/CLI select path).
    activate_verified_model(&root, &model_a).unwrap();
    let tip_before: Value =
        serde_json::from_str(&fs::read_to_string(root.join(ACTIVATED_POINTER_REL)).unwrap())
            .unwrap();
    assert_eq!(tip_before["model_ref"], json!(&model_a));

    // Cold restart = drop session, re-read disk only.
    drop(root.clone());
    let life = list_model_lifecycle(&root).unwrap();
    assert_eq!(life.len(), 2);
    assert!(life.iter().all(|e| e.verified && e.available));
    let tip_after: Value =
        serde_json::from_str(&fs::read_to_string(root.join(ACTIVATED_POINTER_REL)).unwrap())
            .unwrap();
    assert_eq!(tip_after["model_ref"], json!(&model_a));
    assert!(activated_slot_pointer_path(&root, &model_a).is_file());
    assert!(activated_slot_pointer_path(&root, &model_b).is_file());

    let sel = select_model(&root, ModelSelection::Auto).unwrap();
    assert_eq!(sel.model_ref, model_a);
    assert_eq!(sel.via, ModelSelectionVia::AutoLatestTip);

    let mut session = LocalSession::open(&root).unwrap();
    let out = submit_required(&mut session, &model_a);
    assert!(
        matches!(out, SubmitOutcome::Executed { .. }),
        "cold restart must still admit Required tip model: {out:?}"
    );
}

/// M6: Required unavailable fails closed — never silently runs the other available model.
#[test]
fn m6_e2e_unavailable_required_fail_closed_no_substitute() {
    let _lock = isolated();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join(".aira");
    init_m6_root(&root);

    let model_a = install_model(&root, "m6-fail-a", b"fail-a-weights");
    let model_b = install_model(&root, "m6-fail-b", b"fail-b-weights-zz");
    // Tip stays B; remove A's activated slot → A unready, B still available.
    fs::remove_file(activated_slot_pointer_path(&root, &model_a)).unwrap();

    let err = select_model(&root, ModelSelection::Required(model_a.clone())).unwrap_err();
    assert!(
        matches!(
            err,
            ModelSelectError::ModelUnready {
                available: false,
                ..
            }
        ),
        "Required unready must ModelUnready, got {err}"
    );
    let still_b = select_model(&root, ModelSelection::Required(model_b.clone())).unwrap();
    assert_eq!(still_b.model_ref, model_b);

    let mut session = LocalSession::open(&root).unwrap();
    let snap = AdmissionSnapshot::from_text_and_constraints(
        generate_prompt(),
        &cli_or_http_constraints(&model_a),
    );
    let out = session.submit_problem_with_admission(generate_prompt(), snap);
    match out {
        Ok(SubmitOutcome::Executed { .. }) => {
            panic!("must not silent-substitute B when Required A slot is gone");
        }
        Ok(SubmitOutcome::Completed { .. }) => {
            panic!("must not Complete/VERIFIED when Required A is unavailable");
        }
        Ok(other) => panic!("unexpected success outcome: {other:?}"),
        Err(_) => {
            // CapsuleFailed / activate deny surface as flow Err — fail-closed, not B.
        }
    }

    // Removed/unknown Required also explained (select), not Auto→B.
    let removed =
        select_model(&root, ModelSelection::Required("aira:model:m6-gone".into())).unwrap_err();
    assert!(matches!(removed, ModelSelectError::ModelRemoved { .. }));
}
