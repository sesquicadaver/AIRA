//! Local operational plane: problem submit + CSU event drain.

use std::collections::VecDeque;
use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_core::{MemoryObjectStore, ObjectStore};
use aira_csu::support::{json_bytes, local_identity, local_signature, make_artifact, make_event};
use aira_csu::{Csu, CsuRuntime};
use aira_csu_context_basic::ContextBasicCsu;
use aira_csu_evidence_basic::EvidenceBasicCsu;
use aira_csu_execution_basic::ExecutionBasicCsu;
use aira_csu_reduction_basic::ReductionBasicCsu;
use aira_csu_verification_basic::VerificationBasicCsu;
use aira_event::{EventDescriptor, EventSink, EventType, MemoryEventLog};
use aira_object::{AiraRef, ContentHash, ObjectDescriptor, ObjectType, Timestamp};
use serde_json::{json, Value};
use thiserror::Error;

/// Operational flow errors.
#[derive(Debug, Error)]
pub enum FlowError {
    #[error("core: {0}")]
    Core(String),
    #[error("csu: {0}")]
    Csu(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("flow: {0}")]
    Other(String),
}

/// Outcome of submitting a problem statement.
#[derive(Debug, Clone)]
pub enum SubmitOutcome {
    /// Full pipeline produced a verified result.
    Completed {
        problem_id: AiraRef,
        verified_artifact_id: AiraRef,
        result: Value,
    },
    /// Normative alternatives require human collapse (Issue #56).
    NeedsHumanCollapse { field_artifact_id: AiraRef },
}

/// Local in-process operational plane.
pub struct OperationalPlane {
    objects: MemoryObjectStore,
    artifacts: CasArtifactStore,
    events: MemoryEventLog,
    runtime: CsuRuntime,
    problem_ref: Option<AiraRef>,
    seq: u64,
    ready_solutions: Vec<AiraRef>,
}

impl OperationalPlane {
    /// Open a plane rooted at `root` (artifact CAS directory).
    pub fn open(root: impl AsRef<Path>) -> Result<Self, FlowError> {
        Self::open_with_ready(root, vec![])
    }

    /// Open with pre-seeded Ready Solution artifact refs for Reduction reuse.
    pub fn open_with_ready(
        root: impl AsRef<Path>,
        ready_solutions: Vec<AiraRef>,
    ) -> Result<Self, FlowError> {
        let artifacts =
            CasArtifactStore::open(root).map_err(|e| FlowError::Artifact(e.to_string()))?;
        let mut runtime = CsuRuntime::new(local_identity(), local_signature());
        let mut events = MemoryEventLog::new();

        let mut reduction = ReductionBasicCsu::new();
        for id in &ready_solutions {
            reduction = reduction.with_ready_solution(id.clone());
        }

        let handlers: Vec<Box<dyn Csu>> = vec![
            Box::new(ContextBasicCsu::new()),
            Box::new(reduction),
            Box::new(ExecutionBasicCsu::new()),
            Box::new(VerificationBasicCsu::new()),
            Box::new(EvidenceBasicCsu::new()),
        ];
        for h in handlers {
            let id = h.manifest().csu_id.clone();
            runtime
                .register_handler(h, Some(&mut events))
                .map_err(|e| FlowError::Csu(e.to_string()))?;
            runtime
                .activate(&id, Some(&mut events))
                .map_err(|e| FlowError::Csu(e.to_string()))?;
        }

        Ok(Self {
            objects: MemoryObjectStore::new(),
            artifacts,
            events,
            runtime,
            problem_ref: None,
            seq: 1,
            ready_solutions,
        })
    }

    pub fn objects(&self) -> &MemoryObjectStore {
        &self.objects
    }

    pub fn artifacts(&self) -> &CasArtifactStore {
        &self.artifacts
    }

    pub fn artifacts_mut(&mut self) -> &mut CasArtifactStore {
        &mut self.artifacts
    }

    pub fn events(&self) -> &[EventDescriptor] {
        self.events.all()
    }

    pub fn problem_ref(&self) -> Option<&AiraRef> {
        self.problem_ref.as_ref()
    }

    /// Seed a ready solution and rebuild Reduction handler (Issue #54).
    pub fn enable_ready_solution(&mut self, ready_id: AiraRef) -> Result<(), FlowError> {
        self.ready_solutions.push(ready_id.clone());
        let mut reduction = ReductionBasicCsu::new();
        for id in &self.ready_solutions {
            reduction = reduction.with_ready_solution(id.clone());
        }
        let id = reduction.manifest().csu_id.clone();
        // Replace handler map entry while keeping registry Active.
        self.runtime
            .replace_handler(Box::new(reduction))
            .map_err(|e| FlowError::Csu(e.to_string()))?;
        let _ = id;
        Ok(())
    }

    /// Submit a Problem Statement and drain the operational pipeline (#47–#52).
    pub fn submit_problem(&mut self, text: &str) -> Result<SubmitOutcome, FlowError> {
        if is_normative_split(text) {
            return self.emit_differentiated_field(text);
        }

        self.seq += 1;
        let problem_id =
            AiraRef::parse(format!("aira:problem:flow{}", self.seq)).map_err(map_obj)?;
        let hash = ContentHash::sha256_bytes(text.as_bytes());
        let desc = ObjectDescriptor {
            object_id: problem_id.clone(),
            object_type: ObjectType::ProblemStatement,
            schema_version: "0.1".into(),
            created_at: Timestamp::parse("2026-07-10T12:00:00Z").map_err(map_obj)?,
            producer_identity: local_identity(),
            policy_refs: vec![AiraRef::parse("aira:policy:default").map_err(map_obj)?],
            provenance_refs: vec![],
            content_hash: hash,
            signature: local_signature(),
        };
        self.objects
            .create(desc)
            .map_err(|e| FlowError::Core(e.to_string()))?;
        self.problem_ref = Some(problem_id.clone());

        self.seq += 1;
        let ev = make_event(
            &format!("aira:event:psub{}", self.seq),
            EventType::ProblemSubmitted,
            vec![problem_id.clone()],
            vec![],
            vec![],
            Some(text.to_string()),
        );
        self.events
            .append(ev.clone())
            .map_err(|e| FlowError::Other(e.to_string()))?;
        self.drain_from(0)?;

        let verified = self
            .latest_verified_result()
            .ok_or_else(|| FlowError::Other("pipeline produced no verified result".into()))?;
        Ok(SubmitOutcome::Completed {
            problem_id,
            verified_artifact_id: verified.0,
            result: verified.1,
        })
    }

    /// Inject an external event and drain (demos / failure injection).
    pub fn inject_and_drain(&mut self, event: EventDescriptor) -> Result<(), FlowError> {
        let start = self.events.all().len();
        self.events
            .append(event)
            .map_err(|e| FlowError::Other(e.to_string()))?;
        self.drain_from(start)
    }

    pub fn has_evidence_for_results(&self) -> bool {
        self.events()
            .iter()
            .filter(|e| e.event_type == EventType::ResultPublished)
            .any(|rp| {
                self.events().iter().any(|e| {
                    e.event_type == EventType::ArtifactPublished
                        && e.causal_refs.contains(&rp.event_id)
                }) || self.artifacts_of_type(ArtifactType::EvidenceArtifact)
            })
    }

    pub fn has_verified_result_artifact(&self) -> bool {
        self.artifacts_of_type(ArtifactType::VerifiedResultArtifact)
    }

    fn artifacts_of_type(&self, ty: ArtifactType) -> bool {
        // CasArtifactStore has no list API — infer from events' artifact_refs resolve.
        self.events().iter().any(|e| {
            e.artifact_refs.iter().any(|id| {
                self.artifacts
                    .resolve(id)
                    .map(|(d, _)| d.artifact_type == ty)
                    .unwrap_or(false)
            })
        })
    }

    fn latest_verified_result(&self) -> Option<(AiraRef, Value)> {
        for e in self.events().iter().rev() {
            if e.event_type != EventType::ResultPublished
                && e.event_type != EventType::VerificationCompleted
            {
                continue;
            }
            for id in &e.artifact_refs {
                if let Ok((desc, bytes)) = self.artifacts.resolve(id) {
                    if desc.artifact_type == ArtifactType::VerifiedResultArtifact
                        || desc.artifact_type == ArtifactType::ReadySolutionArtifact
                    {
                        if let Ok(v) = serde_json::from_slice::<Value>(&bytes) {
                            return Some((id.clone(), v));
                        }
                    }
                }
            }
        }
        None
    }

    fn drain_from(&mut self, start_idx: usize) -> Result<(), FlowError> {
        let mut queue: VecDeque<EventDescriptor> =
            self.events.all()[start_idx..].iter().cloned().collect();
        let mut guard = 0usize;
        while let Some(ev) = queue.pop_front() {
            guard += 1;
            if guard > 256 {
                return Err(FlowError::Other("event drain exceeded safety limit".into()));
            }
            // Skip pure lifecycle noise for CSU fan-out (still in log).
            if matches!(
                ev.event_type,
                EventType::CSURegistered
                    | EventType::CSUSuspended
                    | EventType::CSUFailed
                    | EventType::PolicyEvaluated
                    | EventType::CustomEvent
            ) {
                continue;
            }
            let before = self.events.all().len();
            self.runtime
                .dispatch_with_artifacts(&ev, &mut self.events, &mut self.artifacts)
                .map_err(|e| FlowError::Csu(e.to_string()))?;
            for neo in self.events.all()[before..].iter().cloned() {
                queue.push_back(neo);
            }
        }
        Ok(())
    }

    fn emit_differentiated_field(&mut self, text: &str) -> Result<SubmitOutcome, FlowError> {
        self.seq += 1;
        let problem_id =
            AiraRef::parse(format!("aira:problem:flow{}", self.seq)).map_err(map_obj)?;
        let hash = ContentHash::sha256_bytes(text.as_bytes());
        let desc = ObjectDescriptor {
            object_id: problem_id.clone(),
            object_type: ObjectType::ProblemStatement,
            schema_version: "0.1".into(),
            created_at: Timestamp::parse("2026-07-10T12:00:00Z").map_err(map_obj)?,
            producer_identity: local_identity(),
            policy_refs: vec![AiraRef::parse("aira:policy:default").map_err(map_obj)?],
            provenance_refs: vec![],
            content_hash: hash,
            signature: local_signature(),
        };
        self.objects
            .create(desc)
            .map_err(|e| FlowError::Core(e.to_string()))?;
        self.problem_ref = Some(problem_id.clone());

        let body = json!({
            "field_type": "DifferentiatedSolutionField",
            "problem_statement_ref": problem_id.as_str(),
            "alternatives": split_alternatives(text),
            "requires_human_collapse": true,
            "auto_collapsed": false
        });
        let payload = json_bytes(&body);
        self.seq += 1;
        let art = make_artifact(
            &format!("aira:artifact:dsf{}", self.seq),
            ArtifactType::OperationalArtifact,
            &payload,
            vec![problem_id],
        );
        let art_id = art.artifact_id.clone();
        self.artifacts
            .publish(art, &payload)
            .map_err(|e| FlowError::Artifact(e.to_string()))?;

        self.seq += 1;
        let ev = make_event(
            &format!("aira:event:dsf{}", self.seq),
            EventType::ProblemSubmitted,
            vec![self.problem_ref.clone().unwrap()],
            vec![art_id.clone()],
            vec![],
            Some("normative_split:requires_human_collapse".into()),
        );
        self.events
            .append(ev)
            .map_err(|e| FlowError::Other(e.to_string()))?;

        Ok(SubmitOutcome::NeedsHumanCollapse {
            field_artifact_id: art_id,
        })
    }
}

fn is_normative_split(text: &str) -> bool {
    let lower = text.to_lowercase();
    (lower.contains(" either ") && lower.contains(" or "))
        || lower.contains(" || ")
        || (lower.contains("-norm") && lower.contains(" or "))
}

fn split_alternatives(text: &str) -> Vec<String> {
    if text.contains(" || ") {
        text.split(" || ").map(|s| s.trim().to_string()).collect()
    } else if let Some(idx) = text.find(" OR ") {
        vec![
            text[..idx].trim().to_string(),
            text[idx + 4..].trim().to_string(),
        ]
    } else if let Some(idx) = text.to_lowercase().find(" or ") {
        vec![
            text[..idx].trim().to_string(),
            text[idx + 4..].trim().to_string(),
        ]
    } else {
        vec![text.to_string()]
    }
}

fn map_obj<E: std::fmt::Display>(e: E) -> FlowError {
    FlowError::Other(e.to_string())
}
