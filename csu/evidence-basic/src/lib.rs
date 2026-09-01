//! Evidence-basic CSU (Issue #45).
//!
//! Observes ResultPublished / CapsuleFailed / VerificationFailed → Evidence artifacts.
//! `#206`: B0-005 Claim vs Assumption — a `claim_kind` of `Claim` without evidence_refs
//! is rejected at OperationalPlane inject/drain (not schema-fixtures-only).

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::AiraRef;
use serde_json::{json, Value};

/// Evidence capture CSU.
pub struct EvidenceBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: String,
}

impl Default for EvidenceBasicCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl EvidenceBasicCsu {
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:evidence.basic",
                "evidence-basic",
                CsuType::Evidence,
                &["ResultPublished", "CapsuleFailed", "VerificationFailed"],
                &["FailureEvidenceCreated", "ArtifactPublished"],
            ),
            seq: 1,
            run_nonce: String::from("0"),
        }
    }

    /// Namespace ids for multi-run local nodes (Epic 8).
    pub fn with_run_nonce(mut self, run_nonce: impl Into<String>) -> Self {
        self.run_nonce = run_nonce.into();
        self
    }

    /// Emit as a distinct publisher identity.
    ///
    /// Requires [`aira_object::register_csu_tenant_signing`] for this CSU before emits.
    pub fn with_publisher(mut self, publisher: AiraRef) -> Self {
        aira_csu::support::apply_publisher(&mut self.manifest, publisher);
        self
    }

    fn next_id(&mut self, kind: &str) -> String {
        let id = format!("aira:{kind}:evi{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }
}

/// Book 0 A5 / B0-005: a Claim coordinate body must carry at least one evidence ref.
///
/// `Assumption` and `Hypothesis` may omit evidence. Bodies without `claim_kind` are not claims.
pub fn claim_lacks_required_evidence(body: &Value) -> bool {
    match body.get("claim_kind").and_then(|v| v.as_str()) {
        Some("Claim") => {
            let Some(arr) = body.get("evidence_refs").and_then(|v| v.as_array()) else {
                return true;
            };
            !arr.iter()
                .any(|r| r.as_str().is_some_and(|s| !s.is_empty()))
        }
        _ => false,
    }
}

impl Csu for EvidenceBasicCsu {
    fn manifest(&self) -> &CsuManifest {
        &self.manifest
    }

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        let is_failure = matches!(
            event.event_type,
            EventType::CapsuleFailed | EventType::VerificationFailed
        );
        let is_result = event.event_type == EventType::ResultPublished;
        if !is_failure && !is_result {
            return Ok(vec![]);
        }

        let body = json!({
            "evidence_kind": if is_failure { "failure" } else { "result" },
            "source_event": event.event_id.as_str(),
            "source_event_type": format!("{:?}", event.event_type),
            "object_refs": event.object_refs.iter().map(|r| r.as_str().to_string()).collect::<Vec<_>>(),
            "artifact_refs": event.artifact_refs.iter().map(|r| r.as_str().to_string()).collect::<Vec<_>>(),
            "note": event.payload_ref.clone().unwrap_or_default(),
            "assigns_final_truth": false
        });
        let payload = json_bytes(&body);
        let aid = self.next_id("artifact");
        let desc = make_artifact_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &aid,
            ArtifactType::EvidenceArtifact,
            &payload,
            vec![event.event_id.clone()],
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.publish_artifact(desc.clone(), &payload)
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        let mut outs = vec![CsuOutput::Artifact {
            descriptor: desc.clone(),
            payload,
        }];

        if is_failure {
            let fe = make_event_as(
                self.manifest.csu_id.clone(),
                self.manifest.publisher_identity.clone(),
                &self.next_id("event"),
                EventType::FailureEvidenceCreated,
                event.object_refs.clone(),
                vec![desc.artifact_id.clone()],
                vec![event.event_id.clone()],
                event.payload_ref.clone(),
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            ctx.append_event(fe.clone()).map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            outs.push(CsuOutput::Event(fe));
        } else {
            let pub_ev = make_event_as(
                self.manifest.csu_id.clone(),
                self.manifest.publisher_identity.clone(),
                &self.next_id("event"),
                EventType::ArtifactPublished,
                event.object_refs.clone(),
                vec![desc.artifact_id.clone()],
                vec![event.event_id.clone()],
                None,
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            ctx.append_event(pub_ev.clone())
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
            outs.push(CsuOutput::Event(pub_ev));
        }

        Ok(outs)
    }
}

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_artifact::CasArtifactStore;
    use aira_csu::support::make_event as mk;
    use aira_event::MemoryEventLog;
    use aira_object::AiraRef;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn failure_creates_failure_evidence() {
        let mut csu = EvidenceBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let ev = mk(
            "aira:event:fail1",
            EventType::VerificationFailed,
            vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            vec![],
            vec![],
            Some("bad output".into()),
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Artifact { descriptor, .. }
                if descriptor.artifact_type == ArtifactType::EvidenceArtifact
        )));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::FailureEvidenceCreated
        )));
    }

    #[test]
    fn claim_without_evidence_lacks_required_evidence() {
        let claim = json!({
            "claim_kind": "Claim",
            "statement": "bare claim",
            "evidence_refs": []
        });
        assert!(claim_lacks_required_evidence(&claim));
        let missing_field = json!({"claim_kind": "Claim", "statement": "no refs field"});
        assert!(claim_lacks_required_evidence(&missing_field));
    }

    #[test]
    fn assumption_and_hypothesis_may_omit_evidence() {
        assert!(!claim_lacks_required_evidence(&json!({
            "claim_kind": "Assumption",
            "evidence_refs": []
        })));
        assert!(!claim_lacks_required_evidence(&json!({
            "claim_kind": "Hypothesis",
            "evidence_refs": []
        })));
        assert!(!claim_lacks_required_evidence(&json!({
            "claim_kind": "Claim",
            "evidence_refs": ["aira:evidence:01EV1"]
        })));
        assert!(!claim_lacks_required_evidence(
            &json!({"result": 4.0, "evidence_refs": []})
        ));
    }
}
