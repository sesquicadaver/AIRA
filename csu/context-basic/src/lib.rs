//! Context-basic CSU (Issue #41).
//!
//! ProblemSubmitted → Context Artifact + ContextResolved.
//! Marks unresolved ambiguity; does not execute or produce results.

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::AiraRef;
use serde_json::json;

/// Deterministic context extraction CSU.
pub struct ContextBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: u64,
}

impl Default for ContextBasicCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextBasicCsu {
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:context.basic",
                "context-basic",
                CsuType::Context,
                &["ProblemSubmitted"],
                &["ContextResolved"],
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
        let id = format!("aira:{kind}:ctx{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }
}

impl Csu for ContextBasicCsu {
    fn manifest(&self) -> &CsuManifest {
        &self.manifest
    }

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        if event.event_type != EventType::ProblemSubmitted {
            return Ok(vec![]);
        }

        let problem_ref = event
            .object_refs
            .first()
            .cloned()
            .unwrap_or_else(|| AiraRef::parse("aira:problem:unknown").expect("ref"));

        let statement = event
            .payload_ref
            .clone()
            .unwrap_or_else(|| "unspecified problem".into());

        let language = if statement.is_ascii() { "en" } else { "und" };
        let mut unresolved = vec!["intent_confidence".to_string()];
        if statement.len() < 3 {
            unresolved.push("underspecified_statement".into());
        }

        let context_body = json!({
            "context_id": format!("aira:context:ctx{}", self.seq),
            "problem_statement_ref": problem_ref.as_str(),
            "context_type": "execution",
            "resolved_factors": {
                "language": language,
                "explicit_constraints": [],
                "statement_preview": statement.chars().take(120).collect::<String>()
            },
            "unresolved_factors": unresolved,
            "confidence": 0.4,
            "scope": { "scope_type": "local", "description": "context-basic" },
            "evidence_refs": [],
            "provenance_refs": [event.event_id.as_str()]
        });
        let payload = json_bytes(&context_body);
        let art_id = self.next_id("artifact");
        let desc = make_artifact_as(
            self.manifest.publisher_identity.clone(),
            &art_id,
            ArtifactType::ContextArtifact,
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

        let ev_id = self.next_id("event");
        let out_ev = make_event_as(
            self.manifest.publisher_identity.clone(),
            &ev_id,
            EventType::ContextResolved,
            vec![problem_ref],
            vec![desc.artifact_id.clone()],
            vec![event.event_id.clone()],
            Some(statement),
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.append_event(out_ev.clone())
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        Ok(vec![
            CsuOutput::Artifact {
                descriptor: desc,
                payload,
            },
            CsuOutput::Event(out_ev),
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
    use aira_artifact::CasArtifactStore;
    use aira_csu::support::make_event as mk;
    use aira_event::MemoryEventLog;
    use aira_object::AiraRef;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn problem_submitted_creates_context_not_result() {
        let mut csu = ContextBasicCsu::new();
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
            "aira:event:p1",
            EventType::ProblemSubmitted,
            vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            vec![],
            vec![],
            Some("Calculate 2 + 2".into()),
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        assert!(outs.iter().any(|o| matches!(o, CsuOutput::Artifact { .. })));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::ContextResolved
        )));
        assert!(!outs.iter().any(|o| matches!(
            o,
            CsuOutput::Artifact { descriptor, .. }
                if descriptor.artifact_type == ArtifactType::VerifiedResultArtifact
        )));
        assert!(log
            .all()
            .iter()
            .any(|e| e.event_type == EventType::ContextResolved));
    }
}
