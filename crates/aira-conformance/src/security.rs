//! MVP security baseline tests (Issue #78).

use std::path::Path;

use aira_artifact::{
    is_private_artifact, ArtifactError, ArtifactStore, ArtifactType, CasArtifactStore,
    PRIVATE_ARTIFACT_POLICY,
};
use aira_csu::support::{make_artifact, make_event};
use aira_csu::{Csu, CsuRegistry};
use aira_csu_context_basic::ContextBasicCsu;
use aira_event::{payload_contains_secret, EventError, EventSink, EventType, MemoryEventLog};
use aira_object::AiraRef;

use crate::report::ConformanceProfile;
use crate::runner::{fail, finalize_suite, pass, CaseResult, ConformanceError, SuiteResult};

/// Run security baseline suite and emit a Conformance Report Artifact (profile C1).
pub fn run_security_baseline(
    artifact_root: impl AsRef<Path>,
) -> Result<SuiteResult, ConformanceError> {
    let cases = vec![
        test_unsigned_csu_rejected(),
        test_invalid_csu_signature_rejected(),
        test_unsigned_artifact_rejected(artifact_root.as_ref()),
        test_invalid_artifact_signature_rejected(artifact_root.as_ref()),
        test_private_artifact_denied(artifact_root.as_ref()),
        test_secret_not_in_events(),
    ];
    finalize_suite(ConformanceProfile::C1, cases, artifact_root)
}

fn test_unsigned_csu_rejected() -> CaseResult {
    let id = "sec.unsigned_csu";
    let mut manifest = ContextBasicCsu::new().manifest().clone();
    manifest.signature.signature_value.clear();
    let mut reg = CsuRegistry::new();
    match reg.register(manifest, None) {
        Err(_) => pass(id),
        Ok(_) => fail(id, "unsigned CSU was accepted"),
    }
}

fn test_invalid_csu_signature_rejected() -> CaseResult {
    let id = "sec.invalid_csu_signature";
    let mut manifest = ContextBasicCsu::new().manifest().clone();
    manifest.signature.signature_value = "TESTSIG".into();
    let mut reg = CsuRegistry::new();
    match reg.register(manifest, None) {
        Err(_) => pass(id),
        Ok(_) => fail(id, "TESTSIG CSU was accepted"),
    }
}

fn test_invalid_artifact_signature_rejected(artifact_root: &Path) -> CaseResult {
    let id = "sec.invalid_artifact_signature";
    let dir = artifact_root.join("sec-badsig");
    let mut store = match CasArtifactStore::open(&dir) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let payload = b"badsig";
    let mut desc = make_artifact(
        "aira:artifact:sec_badsig1",
        ArtifactType::EvidenceArtifact,
        payload,
        vec![],
    );
    desc.signature.signature_value = "TESTSIG".into();
    match store.publish(desc, payload) {
        Err(ArtifactError::Unsigned(_)) | Err(ArtifactError::InvalidSignature(_)) => pass(id),
        other => fail(id, format!("expected signature reject, got {other:?}")),
    }
}

fn test_unsigned_artifact_rejected(artifact_root: &Path) -> CaseResult {
    let id = "sec.unsigned_artifact";
    let dir = artifact_root.join("sec-unsigned");
    let mut store = match CasArtifactStore::open(&dir) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let payload = b"unsigned";
    let mut desc = make_artifact(
        "aira:artifact:sec_unsigned1",
        ArtifactType::EvidenceArtifact,
        payload,
        vec![],
    );
    desc.signature.signature_value.clear();
    match store.publish(desc, payload) {
        Err(ArtifactError::Unsigned(_)) => pass(id),
        other => fail(id, format!("expected Unsigned, got {other:?}")),
    }
}

fn test_private_artifact_denied(artifact_root: &Path) -> CaseResult {
    let id = "sec.private_artifact_denied";
    let dir = artifact_root.join("sec-private");
    let mut store = match CasArtifactStore::open(&dir) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let payload = b"top-secret";
    let mut desc = make_artifact(
        "aira:artifact:sec_private1",
        ArtifactType::EvidenceArtifact,
        payload,
        vec![],
    );
    desc.policy_refs = vec![AiraRef::parse(PRIVATE_ARTIFACT_POLICY).unwrap()];
    if let Err(e) = store.publish(desc.clone(), payload) {
        return fail(id, e.to_string());
    }
    if !is_private_artifact(&desc) {
        return fail(id, "descriptor not marked private");
    }
    match store.resolve(&desc.artifact_id) {
        Err(ArtifactError::AccessDenied(_)) => {}
        other => return fail(id, format!("expected AccessDenied, got {other:?}")),
    }
    match store.resolve_with_access(&desc.artifact_id, true) {
        Ok((_, bytes)) if bytes == payload => pass(id),
        other => fail(id, format!("privileged resolve failed: {other:?}")),
    }
}

fn test_secret_not_in_events() -> CaseResult {
    let id = "sec.no_secrets_in_events";
    let mut log = MemoryEventLog::new();
    let mut ev = make_event(
        "aira:event:sec_leak1",
        EventType::CustomEvent,
        vec![],
        vec![],
        vec![],
        Some("password=should-not-leak".into()),
    );
    match log.append(ev.clone()) {
        Err(EventError::SecretMaterial) => {}
        other => return fail(id, format!("expected SecretMaterial, got {other:?}")),
    }
    ev.payload_ref = Some("Calculate 2 + 2".into());
    if let Err(e) = log.append(ev) {
        return fail(id, e.to_string());
    }
    if payload_contains_secret(Some("ed25519_secret=abc")) {
        pass(id)
    } else {
        fail(id, "secret detector missed ed25519_secret=")
    }
}
