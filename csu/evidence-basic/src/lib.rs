//! Evidence-basic CSU (Issue #45).
//!
//! Observes ResultPublished / CapsuleFailed / VerificationFailed → Evidence artifacts.

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::AiraRef;
use serde_json::json;

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
}
