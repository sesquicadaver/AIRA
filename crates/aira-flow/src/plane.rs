//! Reference-local operational plane: problem submit + CSU event drain.
//!
//! **Role (Analyze-86 / QUEUE #51):** [`OperationalPlane`] is the in-process **C1
//! reference/demo** pipeline (basic CSUs, memory object/event stores, CAS artifacts).
//! It is **not** a production event runtime, **not** a scheduler, **not** a
//! distributed runtime, and **not** a federation runtime. See
//! `docs/operational-plane.md`.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_core::{MemoryObjectStore, ObjectStore};
use aira_csu::support::{
    json_bytes, local_identity, local_signature, make_artifact, make_event, mvp_timestamp,
};
use aira_csu::{Csu, CsuRuntime, DISPATCH_POLICY_ACTION};
use aira_csu_context_basic::ContextBasicCsu;
use aira_csu_epistemic_basic::EpistemicBasicCsu;
use aira_csu_evidence_basic::EvidenceBasicCsu;
use aira_csu_execution_basic::ExecutionBasicCsu;
use aira_csu_execution_llm::{
    AlwaysActivated, ExecutionLlmCsu, ModelActivateGate, ProcessBackend, ACTION_GENERATE_LOCAL,
    ACTIVATE_DENIED,
};
use aira_csu_reduction_basic::ReductionBasicCsu;
use aira_csu_verification_basic::VerificationBasicCsu;
use aira_event::{EventDescriptor, EventSink, EventType, MemoryEventLog};
use aira_object::{AiraRef, ContentHash, ObjectDescriptor, ObjectType};
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
    #[error("research artifact rejected as operational input: {0}")]
    ResearchNonOperational(String),
    #[error("claim without evidence rejected as operational input: {0}")]
    EvidencePrimacy(String),
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
    /// Generate-local completed via execution-llm. Not a Verified Result Artifact.
    Executed {
        problem_id: AiraRef,
        execution_artifact_id: AiraRef,
        result: Value,
    },
    /// Normative alternatives require human collapse (Issue #56).
    NeedsHumanCollapse { field_artifact_id: AiraRef },
}

/// Phase D activate handle: presence of `models/activated.latest.json`.
///
/// Lives on the plane (not in execution-llm) so CSU ↛ CSU holds. Does not
/// mutate inventory or download weights.
#[derive(Debug, Clone)]
pub struct ActivatedPointerGate {
    pointer_path: PathBuf,
}

impl ActivatedPointerGate {
    /// Pointer path relative to an `.aira` (or equivalent) root.
    pub fn from_aira_root(root: impl AsRef<Path>) -> Self {
        Self {
            pointer_path: root.as_ref().join("models/activated.latest.json"),
        }
    }
}

impl ModelActivateGate for ActivatedPointerGate {
    fn check_activated(
        &self,
        payload: &aira_csu_execution_llm::GenerateLocalPayload,
    ) -> Result<(), String> {
        if !self.pointer_path.is_file() {
            return Err(ACTIVATE_DENIED.into());
        }
        let raw =
            std::fs::read_to_string(&self.pointer_path).map_err(|_| ACTIVATE_DENIED.to_string())?;
        let v: Value = serde_json::from_str(&raw).map_err(|_| {
            "activated pointer is not valid JSON (fail-closed; not VERIFIED)".to_string()
        })?;
        let model_ref = v.get("model_ref").and_then(|x| x.as_str()).unwrap_or("");
        if model_ref.is_empty() {
            return Err(ACTIVATE_DENIED.into());
        }
        if let Some(want) = &payload.model_artifact_ref {
            if want.as_str() != model_ref {
                return Err(format!(
                    "model {} is not Phase D activated (activated {model_ref}; fail-closed; not VERIFIED)",
                    want.as_str()
                ));
            }
        }
        Ok(())
    }
}

/// Local in-process C1 reference plane (demo / conformance), not production runtime.
pub struct OperationalPlane {
    objects: MemoryObjectStore,
    artifacts: CasArtifactStore,
    events: MemoryEventLog,
    runtime: CsuRuntime,
    problem_ref: Option<AiraRef>,
    seq: u64,
    run_nonce: String,
    ready_solutions: Vec<AiraRef>,
    /// Durable reuse catalog (`reuse-index.json`). Bound at open; consulted on submit (#204).
    reuse_index: Option<PathBuf>,
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
        Self::open_with_ready_nonce(root, ready_solutions, "0")
    }

    /// Open with a run nonce so artifact/event ids do not collide across local submits.
    pub fn open_with_ready_nonce(
        root: impl AsRef<Path>,
        ready_solutions: Vec<AiraRef>,
        run_nonce: impl Into<String>,
    ) -> Result<Self, FlowError> {
        let run_nonce = run_nonce.into();
        let artifacts =
            CasArtifactStore::open(root).map_err(|e| FlowError::Artifact(e.to_string()))?;
        let mut runtime = CsuRuntime::new(local_identity(), local_signature());
        runtime.bind_policy_gate_from_signer();
        runtime
            .policy_gate_mut()
            .unwrap()
            .allow_action(DISPATCH_POLICY_ACTION);
        let mut events = MemoryEventLog::new();

        let mut reduction = ReductionBasicCsu::new().with_run_nonce(run_nonce.clone());
        for id in &ready_solutions {
            reduction = reduction.with_ready_solution(id.clone());
        }

        let handlers: Vec<Box<dyn Csu>> = vec![
            Box::new(ContextBasicCsu::new().with_run_nonce(run_nonce.clone())),
            Box::new(reduction),
            Box::new(ExecutionBasicCsu::new().with_run_nonce(run_nonce.clone())),
            Box::new(
                ExecutionLlmCsu::new()
                    .with_run_nonce(run_nonce.clone())
                    .with_mock_backend(),
            ),
            Box::new(VerificationBasicCsu::new().with_run_nonce(run_nonce.clone())),
            Box::new(EvidenceBasicCsu::new().with_run_nonce(run_nonce.clone())),
            Box::new(EpistemicBasicCsu::new().with_run_nonce(run_nonce.clone())),
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
            run_nonce,
            ready_solutions,
            reuse_index: None,
        })
    }

    /// Open with a durable reuse-index catalog. Reduction consults it on submit (#204).
    ///
    /// Does not require [`Self::enable_ready_solution`].
    pub fn open_with_reuse_index(
        root: impl AsRef<Path>,
        reuse_index: impl AsRef<Path>,
    ) -> Result<Self, FlowError> {
        Self::open_with_reuse_index_nonce(root, reuse_index, "0")
    }

    /// [`Self::open_with_reuse_index`] plus a run nonce.
    pub fn open_with_reuse_index_nonce(
        root: impl AsRef<Path>,
        reuse_index: impl AsRef<Path>,
        run_nonce: impl Into<String>,
    ) -> Result<Self, FlowError> {
        let mut plane = Self::open_with_ready_nonce(root, vec![], run_nonce)?;
        plane.reuse_index = Some(reuse_index.as_ref().to_path_buf());
        Ok(plane)
    }

    /// Bind a Phase D activate handle on the registered execution-llm CSU.
    ///
    /// Default construction is fail-closed (no gate). Tests inject
    /// [`AlwaysActivated`]; LocalSession injects [`ActivatedPointerGate`].
    pub fn bind_activate_gate(
        &mut self,
        gate: impl ModelActivateGate + 'static,
    ) -> Result<(), FlowError> {
        let csu = ExecutionLlmCsu::new()
            .with_run_nonce(self.run_nonce.clone())
            .with_mock_backend()
            .with_activate_gate(gate);
        self.runtime
            .replace_handler(Box::new(csu))
            .map_err(|e| FlowError::Csu(e.to_string()))?;
        Ok(())
    }

    /// Test/CI double: MockBackend + Phase D activated.
    pub fn enable_activated_mock_llm(&mut self) -> Result<(), FlowError> {
        self.bind_activate_gate(AlwaysActivated)
    }

    /// Bind [`ActivatedPointerGate`] for `models/activated.latest.json` under `aira_root`.
    pub fn bind_phase_d_activate_from_root(
        &mut self,
        aira_root: impl AsRef<Path>,
    ) -> Result<(), FlowError> {
        self.bind_activate_gate(ActivatedPointerGate::from_aira_root(aira_root))
    }

    /// Opt-in process CLI backend. Default [`Self::open`] keeps [`aira_csu_execution_llm::MockBackend`].
    pub fn bind_process_backend(
        &mut self,
        backend: ProcessBackend,
        gate: impl ModelActivateGate + 'static,
    ) -> Result<(), FlowError> {
        let csu = ExecutionLlmCsu::new()
            .with_run_nonce(self.run_nonce.clone())
            .with_process_backend(backend)
            .with_activate_gate(gate);
        self.runtime
            .replace_handler(Box::new(csu))
            .map_err(|e| FlowError::Csu(e.to_string()))?;
        Ok(())
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
    ///
    /// In-memory pre-seed. Durable catalog bind is [`Self::open_with_reuse_index`] (#204).
    pub fn enable_ready_solution(&mut self, ready_id: AiraRef) -> Result<(), FlowError> {
        self.ready_solutions.push(ready_id.clone());
        let mut reduction = ReductionBasicCsu::new().with_run_nonce(self.run_nonce.clone());
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

    /// Bind Reduction from the durable reuse-index for this problem text (#204).
    fn bind_catalog_for_text(&mut self, text: &str) -> Result<(), FlowError> {
        let Some(path) = &self.reuse_index else {
            return Ok(());
        };
        let Some(id_str) =
            crate::reuse::lookup_artifact_id(path, text).map_err(FlowError::Other)?
        else {
            return Ok(());
        };
        let id = AiraRef::parse(&id_str).map_err(map_obj)?;
        if self.artifacts.resolve(&id).is_ok() {
            self.enable_ready_solution(id)?;
        }
        Ok(())
    }

    /// Submit a Problem Statement and drain the operational pipeline (#47–#52).
    pub fn submit_problem(&mut self, text: &str) -> Result<SubmitOutcome, FlowError> {
        if is_normative_split(text) {
            return self.emit_differentiated_field(text);
        }
        self.bind_catalog_for_text(text)?;

        self.seq += 1;
        let problem_id =
            AiraRef::parse(format!("aira:problem:flow{}_{}", self.run_nonce, self.seq))
                .map_err(map_obj)?;
        let hash = ContentHash::sha256_bytes(text.as_bytes());
        let desc = ObjectDescriptor {
            object_id: problem_id.clone(),
            object_type: ObjectType::ProblemStatement,
            schema_version: "0.1".into(),
            created_at: mvp_timestamp(),
            producer_identity: local_identity(),
            policy_refs: vec![AiraRef::parse("aira:policy:default").map_err(map_obj)?],
            provenance_refs: vec![],
            content_hash: hash,
            signature: local_signature(),
        }
        .attach_canonical_signature()
        .map_err(|e| FlowError::Core(e.to_string()))?;
        self.objects
            .create(desc)
            .map_err(|e| FlowError::Core(e.to_string()))?;
        self.problem_ref = Some(problem_id.clone());

        self.seq += 1;
        let ev = make_event(
            &format!("aira:event:psub{}_{}", self.run_nonce, self.seq),
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

        if let Some(verified) = self.latest_verified_result() {
            // C1 2+2 (and any Completed submit) must emit epistemic-assessment (#207).
            // Not a full Epistemic plane.
            let _ = self.latest_epistemic_assessment().ok_or_else(|| {
                FlowError::Other("pipeline produced no epistemic assessment".into())
            })?;
            return Ok(SubmitOutcome::Completed {
                problem_id,
                verified_artifact_id: verified.0,
                result: verified.1,
            });
        }
        if let Some((execution_artifact_id, result)) = self.latest_generate_local_output() {
            return Ok(SubmitOutcome::Executed {
                problem_id,
                execution_artifact_id,
                result,
            });
        }
        Err(FlowError::Other(
            "pipeline produced no verified result".into(),
        ))
    }

    /// Inject an external event and drain (demos / failure injection).
    /// Research / promotion-candidate input is rejected before append (#179).
    /// `claim_kind: Claim` without evidence_refs is rejected before append (#206).
    pub fn inject_and_drain(&mut self, event: EventDescriptor) -> Result<(), FlowError> {
        self.reject_research_as_operational(&event)?;
        self.reject_claim_without_evidence(&event)?;
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

    /// Latest epistemic assessment artifact from the reference plane (#147).
    pub fn latest_epistemic_assessment(&self) -> Option<(AiraRef, Value)> {
        for e in self.events().iter().rev() {
            if e.event_type != EventType::ArtifactPublished {
                continue;
            }
            for id in &e.artifact_refs {
                if let Ok((desc, bytes)) = self.artifacts.resolve(id) {
                    if desc.artifact_type != ArtifactType::KnowledgeArtifact {
                        continue;
                    }
                    if let Ok(v) = serde_json::from_slice::<Value>(&bytes) {
                        if v.get("epistemic_status").is_some()
                            && v.get("assessment_id").is_some()
                            && v.get("confidence").is_some()
                            && v.get("scope").is_some()
                        {
                            return Some((id.clone(), v));
                        }
                    }
                }
            }
        }
        None
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

    /// Latest generate-local ExecutionArtifact from execution-llm (not a VRA).
    pub fn latest_generate_local_output(&self) -> Option<(AiraRef, Value)> {
        for e in self.events().iter().rev() {
            if e.event_type != EventType::CapsuleCompleted {
                continue;
            }
            for id in &e.artifact_refs {
                if let Ok((desc, bytes)) = self.artifacts.resolve(id) {
                    if desc.artifact_type != ArtifactType::ExecutionArtifact {
                        continue;
                    }
                    if let Ok(v) = serde_json::from_slice::<Value>(&bytes) {
                        if v.get("action").and_then(|a| a.as_str()) == Some(ACTION_GENERATE_LOCAL) {
                            return Some((id.clone(), v));
                        }
                    }
                }
            }
        }
        None
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

    /// In-process fan-out for the reference plane (demo safety bound 256, not a scheduler).
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
            self.reject_research_as_operational(&ev)?;
            self.reject_claim_without_evidence(&ev)?;
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
            AiraRef::parse(format!("aira:problem:flow{}_{}", self.run_nonce, self.seq))
                .map_err(map_obj)?;
        let hash = ContentHash::sha256_bytes(text.as_bytes());
        let desc = ObjectDescriptor {
            object_id: problem_id.clone(),
            object_type: ObjectType::ProblemStatement,
            schema_version: "0.1".into(),
            created_at: mvp_timestamp(),
            producer_identity: local_identity(),
            policy_refs: vec![AiraRef::parse("aira:policy:default").map_err(map_obj)?],
            provenance_refs: vec![],
            content_hash: hash,
            signature: local_signature(),
        }
        .attach_canonical_signature()
        .map_err(|e| FlowError::Core(e.to_string()))?;
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
            &format!("aira:artifact:dsf{}_{}", self.run_nonce, self.seq),
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
            &format!("aira:event:dsf{}_{}", self.run_nonce, self.seq),
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

    /// Fail-closed: research types and promotion-candidate events are not operational input.
    fn reject_research_as_operational(&self, event: &EventDescriptor) -> Result<(), FlowError> {
        if event.event_type.is_research_until_promoted() {
            return Err(FlowError::ResearchNonOperational(format!(
                "{:?}",
                event.event_type
            )));
        }
        for id in &event.artifact_refs {
            if let Ok((desc, _)) = self.artifacts.resolve(id) {
                // Unresolved refs stay operational (missing-artifact failure paths).
                if desc.artifact_type.is_research_until_promoted() {
                    return Err(FlowError::ResearchNonOperational(format!(
                        "{}:{:?}",
                        id.as_str(),
                        desc.artifact_type
                    )));
                }
            }
        }
        Ok(())
    }

    /// Fail-closed B0-005: `claim_kind: Claim` without evidence_refs is not operational input (#206).
    ///
    /// `Assumption` / `Hypothesis` may omit evidence. CAS publish remains allowed (same as research).
    fn reject_claim_without_evidence(&self, event: &EventDescriptor) -> Result<(), FlowError> {
        for id in &event.artifact_refs {
            if let Ok((_, bytes)) = self.artifacts.resolve(id) {
                let Ok(body) = serde_json::from_slice::<Value>(&bytes) else {
                    continue;
                };
                if aira_csu_evidence_basic::claim_lacks_required_evidence(&body) {
                    return Err(FlowError::EvidencePrimacy(id.as_str().into()));
                }
            }
        }
        Ok(())
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
