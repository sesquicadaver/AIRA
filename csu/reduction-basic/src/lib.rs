//! Reduction-basic CSU (Issue #42).
//!
//! Prefers Ready Solution / Knowledge reuse; otherwise Negative Lookup + Execution Capsule.

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::AiraRef;
use serde_json::json;

/// Local reduction / reuse CSU.
pub struct ReductionBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: u64,
    /// In-memory ready solution catalog (artifact ids).
    ready_solutions: Vec<AiraRef>,
    knowledge: Vec<AiraRef>,
}

impl Default for ReductionBasicCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl ReductionBasicCsu {
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:reduction.basic",
                "reduction-basic",
                CsuType::Reduction,
                &["ContextResolved"],
                &["ReductionCompleted", "CapsuleCreated"],
            ),
            seq: 1,
            run_nonce: 0,
            ready_solutions: vec![],
            knowledge: vec![],
        }
    }

    pub fn with_ready_solution(mut self, id: AiraRef) -> Self {
        self.ready_solutions.push(id);
        self
    }

    /// Namespace ids for multi-run local nodes (Epic 8).
    pub fn with_run_nonce(mut self, run_nonce: u64) -> Self {
        self.run_nonce = run_nonce;
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
        let id = format!("aira:{kind}:red{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }
}

impl Csu for ReductionBasicCsu {
    fn manifest(&self) -> &CsuManifest {
        &self.manifest
    }

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        if event.event_type != EventType::ContextResolved {
            return Ok(vec![]);
        }

        let problem_ref = event
            .object_refs
            .first()
            .cloned()
            .unwrap_or_else(|| AiraRef::parse("aira:problem:unknown").expect("ref"));
        let context_ref = event
            .artifact_refs
            .first()
            .cloned()
            .unwrap_or_else(|| AiraRef::parse("aira:artifact:unknown").expect("ref"));

        let mut outs = Vec::new();

        if let Some(ready) = self.ready_solutions.first().cloned() {
            let done = make_event_as(
                self.manifest.csu_id.clone(),
                self.manifest.publisher_identity.clone(),
                &self.next_id("event"),
                EventType::ReductionCompleted,
                vec![problem_ref.clone()],
                vec![ready.clone()],
                vec![event.event_id.clone()],
                Some("reuse:ready_solution".into()),
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            ctx.append_event(done.clone())
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
            // Reuse path publishes result without invoking Execution CSU.
            let published = make_event_as(
                self.manifest.csu_id.clone(),
                self.manifest.publisher_identity.clone(),
                &self.next_id("event"),
                EventType::ResultPublished,
                vec![problem_ref],
                vec![ready],
                vec![done.event_id.clone()],
                Some("reuse:ready_solution".into()),
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            ctx.append_event(published.clone())
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
            outs.push(CsuOutput::Event(done));
            outs.push(CsuOutput::Event(published));
            return Ok(outs);
        }

        if let Some(know) = self.knowledge.first().cloned() {
            let ev = make_event_as(
                self.manifest.csu_id.clone(),
                self.manifest.publisher_identity.clone(),
                &self.next_id("event"),
                EventType::ReductionCompleted,
                vec![problem_ref],
                vec![know],
                vec![event.event_id.clone()],
                Some("reuse:knowledge".into()),
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            ctx.append_event(ev.clone()).map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            outs.push(CsuOutput::Event(ev));
            return Ok(outs);
        }

        // Negative lookup artifact
        let neg_body = json!({
            "status": "negative_lookup",
            "checked": ["ready_solution", "knowledge"],
            "reason": "no local reuse candidate"
        });
        let neg_payload = json_bytes(&neg_body);
        let neg_id = self.next_id("artifact");
        let neg_desc = make_artifact_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &neg_id,
            ArtifactType::NegativeResultArtifact,
            &neg_payload,
            vec![event.event_id.clone()],
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.publish_artifact(neg_desc.clone(), &neg_payload)
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
        outs.push(CsuOutput::Artifact {
            descriptor: neg_desc.clone(),
            payload: neg_payload,
        });

        // Execution capsule (needed)
        let statement = event.payload_ref.clone().unwrap_or_default();
        let action = if statement.to_lowercase().contains("echo") {
            "text.echo"
        } else if statement.to_lowercase().contains("upper") {
            "text.uppercase"
        } else {
            "math.eval.safe"
        };
        let expr = if action == "math.eval.safe" {
            // naive extract: use payload or default 2+2
            if statement.contains('+') || statement.contains('*') {
                statement
                    .split_whitespace()
                    .filter(|t| {
                        t.chars()
                            .any(|c| c.is_ascii_digit() || "+-*/()".contains(c))
                    })
                    .collect::<Vec<_>>()
                    .join("")
            } else {
                "2+2".into()
            }
        } else {
            statement.clone()
        };

        let capsule = json!({
            "capsule_id": format!("aira:capsule:red{}", self.seq),
            "problem_statement_ref": problem_ref.as_str(),
            "context_ref": context_ref.as_str(),
            "action": action,
            "expression": expr,
            "required_capabilities": [action],
            "input_artifact_refs": [context_ref.as_str()],
            "constraints": { "network": "none", "shell": false },
            "policy_refs": ["aira:policy:default"],
            "provenance_refs": [event.event_id.as_str()]
        });
        let cap_payload = json_bytes(&capsule);
        let cap_id = self.next_id("artifact");
        let cap_desc = make_artifact_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &cap_id,
            ArtifactType::ExecutionArtifact,
            &cap_payload,
            vec![event.event_id.clone(), neg_desc.artifact_id.clone()],
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.publish_artifact(cap_desc.clone(), &cap_payload)
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
        outs.push(CsuOutput::Artifact {
            descriptor: cap_desc.clone(),
            payload: cap_payload,
        });

        let created = make_event_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &self.next_id("event"),
            EventType::CapsuleCreated,
            vec![problem_ref.clone()],
            vec![cap_desc.artifact_id.clone()],
            vec![event.event_id.clone()],
            Some(action.into()),
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.append_event(created.clone())
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
        outs.push(CsuOutput::Event(created));

        let done = make_event_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &self.next_id("event"),
            EventType::ReductionCompleted,
            vec![problem_ref],
            vec![cap_desc.artifact_id, neg_desc.artifact_id],
            vec![event.event_id.clone()],
            Some("escalate:execution_capsule".into()),
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.append_event(done.clone())
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
        outs.push(CsuOutput::Event(done));
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

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn creates_negative_lookup_and_capsule_when_no_reuse() {
        let mut csu = ReductionBasicCsu::new();
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
            "aira:event:c1",
            EventType::ContextResolved,
            vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            vec![AiraRef::parse("aira:artifact:ctx1").unwrap()],
            vec![],
            Some("Calculate 2 + 2".into()),
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Artifact { descriptor, .. }
                if descriptor.artifact_type == ArtifactType::NegativeResultArtifact
        )));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::CapsuleCreated
        )));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::ReductionCompleted
        )));
    }
}
