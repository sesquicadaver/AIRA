//! Verification-basic CSU (Issue #44).
//!
//! Distinguishes Output Artifact from Verified Result Artifact.

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_object::AiraRef;
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use serde_json::{json, Value};

/// Deterministic verification CSU.
pub struct VerificationBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: u64,
}

impl Default for VerificationBasicCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationBasicCsu {
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:verification.basic",
                "verification-basic",
                CsuType::Verification,
                &["CapsuleCompleted"],
                &[
                    "VerificationCompleted",
                    "VerificationFailed",
                    "ResultPublished",
                ],
            ),
            seq: 1,
            run_nonce: 0,
        }
    }

    /// Namespace ids for multi-run local nodes (Epic 8).
    pub fn with_run_nonce(mut self, run_nonce: u64) -> Self {
        self.run_nonce = run_nonce;
        self
    }

    /// Emit as a distinct publisher identity (must have a signing key in the process keyring).
    pub fn with_publisher(mut self, publisher: AiraRef) -> Self {
        aira_csu::support::apply_publisher(&mut self.manifest, publisher);
        self
    }


    fn next_id(&mut self, kind: &str) -> String {
        let id = format!("aira:{kind}:ver{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }
}

impl Csu for VerificationBasicCsu {
    fn manifest(&self) -> &CsuManifest {
        &self.manifest
    }

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {

        if event.event_type != EventType::CapsuleCompleted {
            return Ok(vec![]);
        }

        let output_id = event
            .artifact_refs
            .first()
            .cloned()
            .ok_or_else(|| CsuHandlerError {
                message: "CapsuleCompleted missing output artifact".into(),
            })?;

        let (out_desc, bytes) = ctx
            .resolve_artifact(&output_id)
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        // Output Artifact must not already be a Verified Result.
        if out_desc.artifact_type == ArtifactType::VerifiedResultArtifact {
            return self.fail(ctx, event, "output already claimed as verified result");
        }

        let body: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        let action = body.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let ok = match action {
            "math.eval.safe" => body
                .get("result")
                .and_then(|v| v.as_f64())
                .is_some_and(|n| n.is_finite()),
            "text.echo" | "text.uppercase" => body.get("result").and_then(|v| v.as_str()).is_some(),
            _ => false,
        };

        if !ok {
            return self.fail(ctx, event, "verification rejected output");
        }

        let verified = json!({
            "result": body.get("result").cloned().unwrap_or(Value::Null),
            "verification_status": "VERIFIED",
            "confidence": 1.0,
            "scope": { "scope_type": "local", "description": "verification-basic" },
            "source_output_ref": output_id.as_str(),
            "artifact_kind": "VerifiedResultArtifact",
            "evidence_refs": [],
            "provenance_refs": [event.event_id.as_str(), output_id.as_str()]
        });
        let payload = json_bytes(&verified);
        let vid = self.next_id("artifact");
        let vdesc = make_artifact_as(
            self.manifest.publisher_identity.clone(),
            
            &vid,
            ArtifactType::VerifiedResultArtifact,
            &payload,
            vec![event.event_id.clone(), output_id.clone()],
        ).map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.publish_artifact(vdesc.clone(), &payload)
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        let completed = make_event_as(
            self.manifest.publisher_identity.clone(),
            
            &self.next_id("event"),
            EventType::VerificationCompleted,
            event.object_refs.clone(),
            vec![vdesc.artifact_id.clone(), output_id.clone()],
            vec![event.event_id.clone()],
            None,
        ).map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.append_event(completed.clone())
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        let published = make_event_as(
            self.manifest.publisher_identity.clone(),
            
            &self.next_id("event"),
            EventType::ResultPublished,
            event.object_refs.clone(),
            vec![vdesc.artifact_id.clone()],
            vec![completed.event_id.clone()],
            None,
        ).map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.append_event(published.clone())
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        Ok(vec![
            CsuOutput::Artifact {
                descriptor: vdesc,
                payload,
            },
            CsuOutput::Event(completed),
            CsuOutput::Event(published),
        ])
    }
}

impl VerificationBasicCsu {
    fn fail(
        &mut self,
        ctx: &mut CsuExecutionContext<'_, '_>,
        event: &EventDescriptor,
        message: &str,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        let failed = make_event_as(
            self.manifest.publisher_identity.clone(),
            
            &self.next_id("event"),
            EventType::VerificationFailed,
            event.object_refs.clone(),
            event.artifact_refs.clone(),
            vec![event.event_id.clone()],
            Some(message.into()),
        ).map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.append_event(failed.clone())
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
        Ok(vec![
            CsuOutput::Failure {
                message: message.into(),
            },
            CsuOutput::Event(failed),
        ])
    }
}

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_artifact::{ArtifactStore, CasArtifactStore};
    use aira_csu::support::{json_bytes, make_artifact, make_event as mk};
    use aira_event::MemoryEventLog;
    use aira_object::AiraRef;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn verifies_math_output_as_verified_result() {
        let mut csu = VerificationBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let payload = json_bytes(&json!({"action":"math.eval.safe","result":4.0}));
        let out = make_artifact(
            "aira:artifact:out1",
            ArtifactType::ExecutionArtifact,
            &payload,
            vec![],
        );
        let oid = out.artifact_id.clone();
        store.publish(out, &payload).unwrap();
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let ev = mk(
            "aira:event:done1",
            EventType::CapsuleCompleted,
            vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            vec![oid],
            vec![],
            None,
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Artifact { descriptor, .. }
                if descriptor.artifact_type == ArtifactType::VerifiedResultArtifact
        )));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::VerificationCompleted
        )));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::ResultPublished
        )));
    }
}
