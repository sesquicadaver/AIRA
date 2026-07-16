//! C0 conformance suite (Issues #64, #66–#69).

use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_core::{InvariantChecker, InvariantViolation, MemoryObjectStore, ObjectStore};
use aira_csu::support::{local_identity, local_signature, make_artifact};
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
        test_artifact_immutability(artifact_root.as_ref()),
        test_event_causality(artifact_root.as_ref()),
        test_policy_gate(),
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
