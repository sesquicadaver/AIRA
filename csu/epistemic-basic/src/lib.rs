//! Epistemic-basic CSU (QUEUE #146 / EPI-001).
//!
//! Separates Evidence, Confidence, Scope, and Epistemic Status; supports
//! Counter Evidence and Revision History. Does **not** implement a full
//! Epistemic plane (`#147` assessment path / later).

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::AiraRef;
use serde_json::json;

/// Minimal Epistemic CSU for EPI-001 smoke.
pub struct EpistemicBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: u64,
    /// Prior assessment artifact ids (revision history).
    revision_refs: Vec<String>,
}

impl Default for EpistemicBasicCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl EpistemicBasicCsu {
    /// Build with Epistemic type and evidence-oriented subscriptions.
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:epistemic.basic",
                "epistemic-basic",
                CsuType::Epistemic,
                &[
                    "ResultPublished",
                    "FailureEvidenceCreated",
                    "ArtifactPublished",
                ],
                &["ArtifactPublished"],
            ),
            seq: 1,
            run_nonce: 0,
            revision_refs: Vec::new(),
        }
    }

    /// Namespace ids for multi-run local nodes.
    pub fn with_run_nonce(mut self, run_nonce: u64) -> Self {
        self.run_nonce = run_nonce;
        self
    }

    /// Emit as a distinct publisher identity.
    pub fn with_publisher(mut self, publisher: AiraRef) -> Self {
        aira_csu::support::apply_publisher(&mut self.manifest, publisher);
        self
    }

    fn next_id(&mut self, kind: &str) -> String {
        let id = format!("aira:{kind}:epi{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }

    /// Build assessment body: Evidence / Confidence / Scope / Status + counters + revisions.
    fn assessment_body(
        &self,
        assessment_id: &str,
        claim_ref: &str,
        evidence_refs: &[String],
        counter_evidence_refs: &[String],
        epistemic_status: &str,
        confidence: f64,
    ) -> serde_json::Value {
        json!({
            "assessment_id": assessment_id,
            "claim_ref": claim_ref,
            "evidence_refs": evidence_refs,
            "counter_evidence_refs": counter_evidence_refs,
            "epistemic_status": epistemic_status,
            "confidence": confidence,
            "scope": {
                "scope_type": "local",
                "description": "epistemic-basic EPI-001"
            },
            "revision_refs": self.revision_refs,
            "signature": aira_object::local_test_signature(assessment_id.as_bytes())
        })
    }
}

impl Csu for EpistemicBasicCsu {
    fn manifest(&self) -> &CsuManifest {
        &self.manifest
    }

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        let is_result = event.event_type == EventType::ResultPublished;
        let is_failure_ev = event.event_type == EventType::FailureEvidenceCreated;
        let is_artifact = event.event_type == EventType::ArtifactPublished;
        if !is_result && !is_failure_ev && !is_artifact {
            return Ok(vec![]);
        }

        let claim_ref = event
            .object_refs
            .first()
            .map(|r| r.as_str().to_string())
            .unwrap_or_else(|| "aira:claim:unknown".into());

        let evidence_refs: Vec<String> = event
            .artifact_refs
            .iter()
            .map(|r| r.as_str().to_string())
            .collect();

        // Counter-evidence path: failure evidence is recorded separately from supporting evidence.
        let (supporting, counters, status, confidence) = if is_failure_ev {
            (
                Vec::<String>::new(),
                evidence_refs.clone(),
                "Contradicted",
                0.2_f64,
            )
        } else if is_result {
            (evidence_refs.clone(), Vec::new(), "Hypothesis", 0.7_f64)
        } else {
            (evidence_refs.clone(), Vec::new(), "Observation", 0.5_f64)
        };

        let assessment_id = self.next_id("epistemic");
        let body = self.assessment_body(
            &assessment_id,
            &claim_ref,
            &supporting,
            &counters,
            status,
            confidence,
        );
        let payload = json_bytes(&body);
        let aid = self.next_id("artifact");
        let desc = make_artifact_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &aid,
            ArtifactType::KnowledgeArtifact,
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

        self.revision_refs
            .push(desc.artifact_id.as_str().to_string());

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

        Ok(vec![
            CsuOutput::Artifact {
                descriptor: desc,
                payload,
            },
            CsuOutput::Event(pub_ev),
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

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_is_epistemic_type() {
        let csu = EpistemicBasicCsu::new();
        assert_eq!(csu.manifest().csu_type, CsuType::Epistemic);
        assert_eq!(csu.manifest().csu_id.as_str(), "aira:csu:epistemic.basic");
        csu.manifest().validate_for_registration().unwrap();
    }

    /// EPI-001: separate Evidence, Confidence, Scope, Epistemic Status;
    /// Counter Evidence; Revision History.
    #[test]
    fn epi_001_assessment_separates_coordinates_and_revisions() {
        let mut csu = EpistemicBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );

        let problem = AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap();
        let support = AiraRef::parse(
            "aira:artifact:sha256_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap();
        let result_ev = mk(
            "aira:event:epi-result",
            EventType::ResultPublished,
            vec![problem.clone()],
            vec![support],
            vec![],
            None,
        );
        let outs1 = csu.on_event(&result_ev, &mut ctx).unwrap();
        let body1 = outs1
            .iter()
            .find_map(|o| match o {
                CsuOutput::Artifact { payload, .. } => {
                    Some(serde_json::from_slice::<serde_json::Value>(payload).unwrap())
                }
                _ => None,
            })
            .expect("assessment artifact");

        assert!(body1.get("evidence_refs").is_some());
        assert!(body1.get("confidence").is_some());
        assert!(body1.get("scope").is_some());
        assert_eq!(body1["epistemic_status"], json!("Hypothesis"));
        assert!(body1["counter_evidence_refs"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(body1["revision_refs"].as_array().unwrap().is_empty());

        let counter = AiraRef::parse(
            "aira:artifact:sha256_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .unwrap();
        let fail_ev = mk(
            "aira:event:epi-fail",
            EventType::FailureEvidenceCreated,
            vec![problem],
            vec![counter],
            vec![],
            Some("counter".into()),
        );
        let outs2 = csu.on_event(&fail_ev, &mut ctx).unwrap();
        let body2 = outs2
            .iter()
            .find_map(|o| match o {
                CsuOutput::Artifact { payload, .. } => {
                    Some(serde_json::from_slice::<serde_json::Value>(payload).unwrap())
                }
                _ => None,
            })
            .expect("revised assessment");

        assert_eq!(body2["epistemic_status"], json!("Contradicted"));
        assert!(!body2["counter_evidence_refs"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(body2["revision_refs"].as_array().unwrap().len(), 1);

        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:epistemic:assessment:0.1", &body1)
            .unwrap();
        reg.validate("aira:schema:epistemic:assessment:0.1", &body2)
            .unwrap();
    }
}
