//! C0 conformance suite (Issues #64, #66–#69).
//!
//! Event-causality cases may drive [`aira_flow::OperationalPlane`] as the C1
//! **reference/demo** plane (`docs/operational-plane.md`), not a production runtime.

use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_core::{
    InvariantChecker, InvariantViolation, MemoryObjectStore, ObjectStore, SqliteObjectStore,
};
use aira_csu::support::{local_identity, local_signature, make_artifact, make_event};
use aira_csu::{CsuRuntime, DISPATCH_POLICY_ACTION};
use aira_event::{EventType, MemoryEventLog};
use aira_object::{AiraRef, ObjectDescriptor, Timestamp};
use aira_policy::{PolicyDecisionKind, PolicyGate, PolicyQuery};
use aira_schema::SchemaRegistry;

use crate::report::ConformanceProfile;
use crate::runner::{fail, finalize_suite, pass, CaseResult, ConformanceError, SuiteResult};

/// Run the C0 conformance suite and emit a Conformance Report Artifact.
pub fn run_c0(artifact_root: impl AsRef<Path>) -> Result<SuiteResult, ConformanceError> {
    let cases = vec![
        test_ontology_schemas(),
        test_object_immutability(),
        test_handle_opacity(),
        test_object_verify_on_read(),
        test_artifact_immutability(artifact_root.as_ref()),
        test_artifact_verify_on_read(artifact_root.as_ref()),
        test_event_causality(artifact_root.as_ref()),
        test_policy_gate(),
        test_csu_dispatch_policy(),
    ];
    finalize_suite(ConformanceProfile::C0, cases, artifact_root)
}

fn load_registry() -> Result<SchemaRegistry, ConformanceError> {
    let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR"))
        .map_err(|e| ConformanceError::Schema(e.to_string()))?;
    SchemaRegistry::load(root.join("schemas")).map_err(|e| ConformanceError::Schema(e.to_string()))
}

/// Ontology / schema presence for core C0 descriptors.
fn test_ontology_schemas() -> CaseResult {
    let id = "c0.ontology.schemas";
    let reg = match load_registry() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let required = [
        "aira:schema:common:ref:0.1",
        "aira:schema:core:object-descriptor:0.1",
        "aira:schema:event:event-descriptor:0.1",
        "aira:schema:artifact:artifact-descriptor:0.1",
        "aira:schema:policy:decision:0.1",
        "aira:schema:conformance:report:0.1",
    ];
    let ids = reg.list_ids();
    for need in required {
        if !ids.iter().any(|x| x == need) {
            return fail(id, format!("missing schema {need}"));
        }
    }
    let desc = ObjectDescriptor::example_problem();
    let v = match serde_json::to_value(&desc) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = reg.validate("aira:schema:core:object-descriptor:0.1", &v) {
        return fail(id, e.to_string());
    }
    pass(id)
}

/// #66 — in-place object mutation fails + InvariantViolation event.
fn test_object_immutability() -> CaseResult {
    let id = "c0.object.immutability";
    let mut store = MemoryObjectStore::new();
    let desc = ObjectDescriptor::example_problem();
    let handle = match store.create(desc.clone()) {
        Ok(h) => h,
        Err(e) => return fail(id, e.to_string()),
    };
    let mut mutated = desc.clone();
    mutated.schema_version = "0.2".into();
    match store.replace_in_place(&handle, mutated) {
        Err(aira_core::CoreError::Invariant(InvariantViolation::ObjectImmutability { .. })) => {}
        other => return fail(id, format!("expected ObjectImmutability, got {other:?}")),
    }

    let mut log = MemoryEventLog::new();
    let mut checker = InvariantChecker::new(local_identity(), local_signature());
    match checker.on_object_mutation_attempt(&desc.object_id, &mut log) {
        Err(aira_core::CoreError::Invariant(InvariantViolation::ObjectImmutability { .. })) => {}
        other => return fail(id, format!("expected invariant emit err, got {other:?}")),
    }
    if !log
        .all()
        .iter()
        .any(|e| e.event_type == EventType::InvariantViolation)
    {
        return fail(id, "InvariantViolation event missing");
    }
    pass(id)
}

/// B1-003 — CSU cannot infer storage path / internal token from Handle debug output.
fn test_handle_opacity() -> CaseResult {
    let id = "c0.object.handle_opacity";
    let object_ref = match AiraRef::parse("aira:problem:01OPACITY") {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let token = 0xDEADBEEF_u64;
    let handle = aira_object::Handle::new(object_ref.clone(), token);
    let dbg = format!("{handle:?}");
    if dbg.contains(&token.to_string()) {
        return fail(id, "Debug output leaks storage_token numeric value");
    }
    for needle in ["/", "\\", ".aira", "sqlite", "db/", "path"] {
        if dbg.contains(needle) {
            return fail(
                id,
                format!("Debug output leaks path-like substring: {needle}"),
            );
        }
    }
    if !dbg.contains("<opaque>") {
        return fail(id, "Debug output must mark internal token as opaque");
    }
    if handle.object_ref() != &object_ref {
        return fail(id, "object_ref mismatch on CSU-visible handle surface");
    }
    pass(id)
}

/// B1-001 verify-on-read — tampered stored descriptor fails on open / get_by_object_id.
fn test_object_verify_on_read() -> CaseResult {
    let id = "c0.object.verify_on_read";
    let dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => return fail(id, e.to_string()),
    };
    let path = dir.path().join("objects.db");
    let mut store = match SqliteObjectStore::open(&path) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let desc = ObjectDescriptor::example_problem();
    let object_id = desc.object_id.clone();
    let handle = match store.create(desc) {
        Ok(h) => h,
        Err(e) => return fail(id, e.to_string()),
    };

    let conn = match rusqlite::Connection::open(&path) {
        Ok(c) => c,
        Err(e) => return fail(id, e.to_string()),
    };
    let mut tampered = ObjectDescriptor::example_problem();
    tampered.schema_version = "0.2".into();
    let tampered_json = match serde_json::to_string(&tampered) {
        Ok(j) => j,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = conn.execute(
        "UPDATE objects SET descriptor_json = ?1 WHERE object_id = ?2",
        rusqlite::params![tampered_json, object_id.as_str()],
    ) {
        return fail(id, e.to_string());
    }

    let reopened = match SqliteObjectStore::open(&path) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    match reopened.open(&handle) {
        Err(aira_core::CoreError::InvalidSignature(_)) => {}
        other => return fail(id, format!("open expected InvalidSignature, got {other:?}")),
    }
    match reopened.get_by_object_id(&object_id) {
        Err(aira_core::CoreError::InvalidSignature(_)) => {}
        other => {
            return fail(
                id,
                format!("get_by_object_id expected InvalidSignature, got {other:?}"),
            )
        }
    }
    pass(id)
}

/// #67 — artifact content mutation fails + violation event.
fn test_artifact_immutability(artifact_root: &Path) -> CaseResult {
    let id = "c0.artifact.immutability";
    let dir = artifact_root.join("c0-artifact-immut");
    let mut store = match CasArtifactStore::open(&dir) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let payload = b"immutable-payload";
    let desc = make_artifact(
        "aira:artifact:conf_immut1",
        ArtifactType::EvidenceArtifact,
        payload,
        vec![],
    );
    let art_id = desc.artifact_id.clone();
    if let Err(e) = store.publish(desc, payload) {
        return fail(id, e.to_string());
    }
    match store.replace_payload(&art_id, b"mutated") {
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        other => return fail(id, format!("expected Immutable, got {other:?}")),
    }

    let mut log = MemoryEventLog::new();
    let mut checker = InvariantChecker::new(local_identity(), local_signature());
    match checker.on_artifact_mutation_attempt(&art_id, &mut log) {
        Err(aira_core::CoreError::Invariant(InvariantViolation::ArtifactImmutability {
            ..
        })) => {}
        other => {
            return fail(
                id,
                format!("expected artifact invariant emit, got {other:?}"),
            )
        }
    }
    if !log
        .all()
        .iter()
        .any(|e| e.event_type == EventType::InvariantViolation)
    {
        return fail(id, "InvariantViolation/ArtifactInvalid event missing");
    }
    pass(id)
}

/// B1-002 verify-on-read — tampered index / sidecar / CAS bytes fail on resolve.
fn test_artifact_verify_on_read(artifact_root: &Path) -> CaseResult {
    let id = "c0.artifact.verify_on_read";
    let dir = artifact_root.join("c0-artifact-verify");
    let mut store = match CasArtifactStore::open(&dir) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let payload = b"artifact-verify-on-read";
    let desc = make_artifact(
        "aira:artifact:conf_verify1",
        ArtifactType::EvidenceArtifact,
        payload,
        vec![],
    );
    let artifact_id = desc.artifact_id.clone();
    let published = match store.publish(desc.clone(), payload) {
        Ok(p) => p,
        Err(e) => return fail(id, e.to_string()),
    };

    let index_path = dir.join("index.json");
    let raw = match std::fs::read_to_string(&index_path) {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let mut file: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    let artifacts = match file.get_mut("artifacts").and_then(|v| v.as_object_mut()) {
        Some(a) => a,
        None => return fail(id, "index.json missing artifacts map"),
    };
    let key = artifact_id.as_str();
    let entry = match artifacts.get(key) {
        Some(v) => v.clone(),
        None => return fail(id, "artifact missing from index"),
    };
    let mut tampered = entry;
    if let Some(obj) = tampered.as_object_mut() {
        obj.insert(
            "schema_version".into(),
            serde_json::Value::String("0.2".into()),
        );
    } else {
        return fail(id, "index artifact entry is not an object");
    }
    artifacts.insert(key.to_string(), tampered);
    if let Err(e) = std::fs::write(
        &index_path,
        serde_json::to_string_pretty(&file).unwrap_or_default(),
    ) {
        return fail(id, e.to_string());
    }

    let reopened = match CasArtifactStore::open(&dir) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    match reopened.resolve(&artifact_id) {
        Err(aira_artifact::ArtifactError::InvalidSignature(_)) => {}
        other => {
            return fail(
                id,
                format!("resolve expected InvalidSignature, got {other:?}"),
            )
        }
    }

    // Restore index, tamper sidecar.
    if let Err(e) = std::fs::write(&index_path, raw) {
        return fail(id, e.to_string());
    }
    let sidecar_path = published.cas_path.with_extension("json");
    let mut sidecar = desc.clone();
    sidecar.schema_version = "0.2".into();
    if let Err(e) = std::fs::write(
        &sidecar_path,
        serde_json::to_string_pretty(&sidecar).unwrap_or_default(),
    ) {
        return fail(id, e.to_string());
    }
    match store.resolve(&artifact_id) {
        Err(aira_artifact::ArtifactError::InvalidSignature(_)) => {}
        other => {
            return fail(
                id,
                format!("sidecar tamper expected InvalidSignature, got {other:?}"),
            )
        }
    }

    if let Err(e) = std::fs::write(
        &sidecar_path,
        serde_json::to_string_pretty(&desc).unwrap_or_default(),
    ) {
        return fail(id, e.to_string());
    }
    if let Err(e) = std::fs::write(&published.cas_path, b"tampered-cas") {
        return fail(id, e.to_string());
    }
    match store.resolve(&artifact_id) {
        Err(aira_artifact::ArtifactError::HashMismatch { .. }) => {}
        other => {
            return fail(
                id,
                format!("CAS tamper expected HashMismatch, got {other:?}"),
            )
        }
    }
    pass(id)
}

/// #68 — operational event chain + causal_refs preserved.
fn test_event_causality(artifact_root: &Path) -> CaseResult {
    let id = "c0.event.causality";
    let run_dir = artifact_root.join("c0-causality");
    let mut plane = match aira_flow::OperationalPlane::open(&run_dir) {
        Ok(p) => p,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = plane.submit_problem("Calculate 2 + 2") {
        return fail(id, e.to_string());
    }
    let events = plane.events();
    let required = [
        EventType::ProblemSubmitted,
        EventType::ContextResolved,
        EventType::CapsuleCreated,
        EventType::CapsuleCompleted,
        EventType::ResultPublished,
    ];
    for need in required {
        if !events.iter().any(|e| e.event_type == need) {
            return fail(id, format!("missing event {need:?}"));
        }
    }
    let with_causal = events.iter().filter(|e| !e.causal_refs.is_empty()).count();
    if with_causal == 0 {
        let has_problem_ref = events
            .iter()
            .any(|e| e.event_type == EventType::ProblemSubmitted && !e.object_refs.is_empty());
        if !has_problem_ref {
            return fail(
                id,
                "causal_refs empty and ProblemSubmitted lacks object_refs",
            );
        }
    }
    if let Some(ev) = events
        .iter()
        .find(|e| e.event_type == EventType::CapsuleCompleted)
    {
        if ev.artifact_refs.is_empty() && ev.causal_refs.is_empty() {
            return fail(id, "CapsuleCompleted missing refs");
        }
    }
    pass(id)
}

/// #69 — policy gate rejects uncontrolled action; enum limited.
fn test_policy_gate() -> CaseResult {
    let id = "c0.policy.gate";
    let mut log = MemoryEventLog::new();
    let mut gate = PolicyGate::new(local_signature());
    let mk = |action: &str| PolicyQuery {
        subject: AiraRef::parse("aira:csu:conf.test").unwrap(),
        csu_ref: Some("aira:csu:conf.test".into()),
        action: action.into(),
        object_refs: vec![],
        artifact_refs: vec![],
        context_refs: vec![],
        evidence_refs: vec![],
        requested_at: Timestamp::parse("2026-07-10T12:00:00Z").unwrap(),
    };

    let decision = match gate.check(mk("unlisted_controlled_action"), Some(&mut log)) {
        Ok(d) => d,
        Err(e) => return fail(id, e.to_string()),
    };
    if decision.decision != PolicyDecisionKind::Deny {
        return fail(
            id,
            format!(
                "expected DENY for unlisted action, got {:?}",
                decision.decision
            ),
        );
    }

    gate.allow_action("ok_read");
    gate.require_action("needs_more");
    let allow = match gate.check(mk("ok_read"), Some(&mut log)) {
        Ok(d) => d,
        Err(e) => return fail(id, e.to_string()),
    };
    let require = match gate.check(mk("needs_more"), Some(&mut log)) {
        Ok(d) => d,
        Err(e) => return fail(id, e.to_string()),
    };
    if allow.decision != PolicyDecisionKind::Allow {
        return fail(id, "ALLOW path broken");
    }
    if require.decision != PolicyDecisionKind::Require {
        return fail(id, "REQUIRE path broken");
    }
    for kind in [
        PolicyDecisionKind::Allow,
        PolicyDecisionKind::Deny,
        PolicyDecisionKind::Require,
    ] {
        let v = serde_json::to_value(kind).unwrap();
        let s = v.as_str().unwrap_or("");
        if !matches!(s, "ALLOW" | "DENY" | "REQUIRE") {
            return fail(id, format!("unexpected decision encoding {s}"));
        }
    }
    pass(id)
}

/// B1-006 — dispatch fail-closed without bound policy gate or ALLOW on `csu.dispatch`.
fn test_csu_dispatch_policy() -> CaseResult {
    let id = "c0.csu.dispatch_policy";
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct EchoCsu {
        manifest: aira_csu::CsuManifest,
        received: Arc<AtomicUsize>,
    }

    impl aira_csu::Csu for EchoCsu {
        fn manifest(&self) -> &aira_csu::CsuManifest {
            &self.manifest
        }

        fn on_event(
            &mut self,
            _event: &aira_event::EventDescriptor,
            _ctx: &mut aira_csu::CsuExecutionContext<'_, '_>,
        ) -> Result<Vec<aira_csu::CsuOutput>, aira_csu::CsuHandlerError> {
            self.received.fetch_add(1, Ordering::SeqCst);
            Ok(vec![])
        }
    }

    let mut log = MemoryEventLog::new();
    let mut rt = CsuRuntime::new(local_identity(), local_signature());
    let mut manifest = aira_csu::support::basic_manifest(
        "aira:csu:conf.dispatch",
        "dispatch-echo",
        aira_csu::CsuType::Execution,
        &[],
        &[],
    );
    manifest.event_subscriptions = vec![serde_json::json!({"event_type": "ProblemSubmitted"})];
    if let Err(e) = manifest.resign_canonical() {
        return fail(id, e.to_string());
    }
    let csu_id = manifest.csu_id.clone();
    let received = Arc::new(AtomicUsize::new(0));
    if let Err(e) = rt.register_handler(
        Box::new(EchoCsu {
            manifest,
            received: received.clone(),
        }),
        Some(&mut log),
    ) {
        return fail(id, e.to_string());
    }
    if let Err(e) = rt.activate(&csu_id, Some(&mut log)) {
        return fail(id, e.to_string());
    }

    let ev = make_event(
        "aira:event:conf_dispatch1",
        EventType::ProblemSubmitted,
        vec![],
        vec![],
        vec![],
        None,
    );
    match rt.dispatch(&ev, &mut log) {
        Err(aira_csu::CsuError::Isolation(_)) => {}
        other => {
            return fail(
                id,
                format!("expected Isolation without gate, got {other:?}"),
            )
        }
    }
    if received.load(Ordering::SeqCst) != 0 {
        return fail(id, "CSU invoked without policy gate");
    }

    rt.bind_policy_gate(PolicyGate::new(local_signature()));
    match rt.dispatch(&ev, &mut log) {
        Err(aira_csu::CsuError::Dispatch(_)) => {}
        other => return fail(id, format!("expected Dispatch on DENY, got {other:?}")),
    }
    if received.load(Ordering::SeqCst) != 0 {
        return fail(id, "CSU invoked on policy DENY");
    }

    rt.policy_gate_mut()
        .unwrap()
        .allow_action(DISPATCH_POLICY_ACTION);
    if let Err(e) = rt.dispatch(&ev, &mut log) {
        return fail(id, e.to_string());
    }
    if received.load(Ordering::SeqCst) != 1 {
        return fail(id, "CSU not invoked after ALLOW");
    }
    pass(id)
}
