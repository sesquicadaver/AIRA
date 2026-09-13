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
    run_nonce: String,
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

        // #324 / RFC-0209 + #335 / RFC-0219: pull immutable admission snapshot;
        // present factor must pass kind/schema boundary.
        let mut admission_snapshot = None;
        for id in &event.artifact_refs {
            if let Ok((_desc, bytes)) = ctx.resolve_artifact(id) {
                if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    if v.get("kind").and_then(|k| k.as_str()) == Some("admission_snapshot") {
                        let schema = v
                            .get("payload_schema")
                            .and_then(|s| s.as_str())
                            .unwrap_or("");
                        if schema != "aira:schema:admission:snapshot:0.1" {
                            return Err(CsuHandlerError {
                                message: format!(
                                    "admission boundary: context factor payload_schema must be aira:schema:admission:snapshot:0.1, got {schema}"
                                ),
                            });
                        }
                        admission_snapshot = Some(v);
                        break;
                    }
                }
            }
        }

        let context_body = json!({
            "context_id": format!("aira:context:ctx{}", self.seq),
            "problem_statement_ref": problem_ref.as_str(),
            "context_type": "execution",
            "resolved_factors": {
                "language": language,
                "explicit_constraints": [],
                "statement_preview": statement.chars().take(120).collect::<String>(),
                "admission_snapshot": admission_snapshot
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
            self.manifest.csu_id.clone(),
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
            self.manifest.csu_id.clone(),
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
    use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
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

    #[test]
    fn problem_submitted_with_admission_snapshot_in_context() {
        use aira_csu::support::{json_bytes, make_artifact};
        use aira_object::ContentHash;

        let mut csu = ContextBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();

        let snap = json!({
            "payload_schema": "aira:schema:admission:snapshot:0.1",
            "kind": "admission_snapshot",
            "statement_content_hash": ContentHash::sha256_bytes(b"Calculate 2 + 2").as_str(),
            "generation": {},
            "placement": "local",
            "resource_budget": {},
            "fallback": {
                "allow_model_fallback": false,
                "allow_placement_fallback": false
            },
            "reuse_policy": "allow_reuse"
        });
        let bytes = json_bytes(&snap);
        let art = make_artifact(
            "aira:artifact:admit-test",
            ArtifactType::OperationalArtifact,
            &bytes,
            vec![],
        );
        let art_id = art.artifact_id.clone();
        store.publish(art, &bytes).unwrap();

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
            vec![art_id],
            vec![],
            Some("Calculate 2 + 2".into()),
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        let body = outs
            .iter()
            .find_map(|o| match o {
                CsuOutput::Artifact { payload, .. } => {
                    serde_json::from_slice::<serde_json::Value>(payload).ok()
                }
                _ => None,
            })
            .expect("context artifact");
        let admit = body
            .pointer("/resolved_factors/admission_snapshot")
            .expect("admission_snapshot factor");
        assert_eq!(
            admit.get("kind").and_then(|k| k.as_str()),
            Some("admission_snapshot")
        );
    }

    #[test]
    fn problem_submitted_rejects_admission_with_wrong_schema() {
        use aira_csu::support::{json_bytes, make_artifact};
        use aira_object::ContentHash;

        let mut csu = ContextBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();

        let snap = json!({
            "payload_schema": "aira:schema:forged",
            "kind": "admission_snapshot",
            "statement_content_hash": ContentHash::sha256_bytes(b"Calculate 2 + 2").as_str(),
        });
        let bytes = json_bytes(&snap);
        let art = make_artifact(
            "aira:artifact:admit-bad",
            ArtifactType::OperationalArtifact,
            &bytes,
            vec![],
        );
        let art_id = art.artifact_id.clone();
        store.publish(art, &bytes).unwrap();

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
            vec![art_id],
            vec![],
            Some("Calculate 2 + 2".into()),
        );
        let err = csu.on_event(&ev, &mut ctx).expect_err("bad schema");
        assert!(
            err.message.contains("admission boundary"),
            "{}",
            err.message
        );
    }
}
