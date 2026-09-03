//! Execution-llm CSU (QUEUE #211 / Analyze-246; plane `#213`; activate gate `#214`;
//! process backend `#215`; child env whitelist `#219`; bounded pipes `#220`;
//! network=none contract `#222`).
//!
//! Host-local `text.generate.local` capsules complete only through a bound
//! [`GenerateBackend`] **and** a bound [`ModelActivateGate`]. [`MockBackend`] is
//! deterministic and never shells out or uses the network. [`ProcessBackend`]
//! spawns a fixed argv (no shell) and fail-closes when the binary is
//! missing. Missing backend, missing/inactive Phase D activate, or invalid
//! payload → [`EventType::CapsuleFailed`], never a fake VERIFIED result.
//!
//! OperationalPlane registers this CSU with [`MockBackend`] (`#213`) and injects
//! the activate handle (`#214`). Capsules whose action is not generate-local are
//! skipped so fan-out with execution-basic does not fail C1 `math.eval.safe`.
//! Process backend is selectable; default plane/CI stay mock. Child spawn uses
//! `env_clear` plus PATH/HOME/LANG (`#219`). stdout/stderr are capped during read
//! (`#220`). `network=none` is AIRA-mediated (`#222` / RFC-0116): the adapter
//! opens no sockets; the child is not an OS network-off sandbox. No Cargo dep on
//! inventory/acquisition CSUs.

mod process;

pub use process::{
    backend_kind_from, backend_kind_from_env, ProcessBackend, CHILD_ENV_ALLOWLIST, EMPTY_STDOUT,
    ENV_LLM_BACKEND, ENV_PROCESS_ARGS, ENV_PROCESS_BIN, ENV_PROCESS_TIMEOUT_MS, MISSING_BINARY,
    NETWORK_NONE_CONTRACT, NONZERO_EXIT, PIPE_OVERFLOW, PIPE_STDERR_LIMIT, PIPE_STDOUT_LIMIT,
    PROCESS_BACKEND_ID, SPAWN_FAILED, TIMED_OUT,
};

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::{AiraRef, Signature};
use serde::Deserialize;
use serde_json::{json, Value};

/// Payload `$id` from RFC-0105 / `aira:schema:execution:generate-local:0.1`.
pub const PAYLOAD_SCHEMA_ID: &str = "aira:schema:execution:generate-local:0.1";

/// Capsule action this CSU accepts.
pub const ACTION_GENERATE_LOCAL: &str = "text.generate.local";

/// Mock backend identifier stamped on successful output.
pub const MOCK_BACKEND_ID: &str = "mock";

/// Fail-closed activate-gate message. Not a VERIFIED result.
pub const ACTIVATE_DENIED: &str = "model is not Phase D activated (fail-closed; not VERIFIED)";

/// Constraints frozen by the generate-local schema.
///
/// `network=none` is AIRA-mediated ([`NETWORK_NONE_CONTRACT`]): the adapter
/// opens no sockets. It is not an OS sandbox.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateLocalConstraints {
    pub network: String,
    pub shell: bool,
}

/// Strict generate-local payload (RFC-0105). Extra JSON properties fail deserialize.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateLocalPayload {
    pub payload_schema: String,
    pub action: String,
    pub prompt: String,
    #[serde(default)]
    pub problem_statement_ref: Option<AiraRef>,
    #[serde(default)]
    pub model_artifact_ref: Option<AiraRef>,
    pub constraints: GenerateLocalConstraints,
    pub provenance_refs: Vec<AiraRef>,
    pub signature: Signature,
}

impl GenerateLocalPayload {
    /// Fail closed on schema id, action const, empty prompt, or relaxed constraints.
    pub fn validate(&self) -> Result<(), String> {
        if self.payload_schema != PAYLOAD_SCHEMA_ID {
            return Err(format!(
                "payload_schema must be {PAYLOAD_SCHEMA_ID}, got {}",
                self.payload_schema
            ));
        }
        if self.action != ACTION_GENERATE_LOCAL {
            return Err(format!(
                "unsupported action: {} (want {ACTION_GENERATE_LOCAL})",
                self.action
            ));
        }
        if self.prompt.is_empty() {
            return Err("prompt must be non-empty".into());
        }
        if self.constraints.network != "none" {
            return Err(format!(
                "network access forbidden ({NETWORK_NONE_CONTRACT})"
            ));
        }
        if self.constraints.shell {
            return Err("shell execution forbidden".into());
        }
        Ok(())
    }
}

/// Local generate backend.
///
/// [`MockBackend`] is in-process (no spawn, no sockets). [`ProcessBackend`]
/// spawns a **fixed argv** (no shell; never `sh -c`).
pub trait GenerateBackend: Send {
    fn generate(&self, payload: &GenerateLocalPayload) -> Result<Value, String>;
}

/// Deterministic in-process backend. No model, no process, no sockets.
#[derive(Debug, Default, Clone, Copy)]
pub struct MockBackend;

impl MockBackend {
    /// Stable mock text for a prompt (tests and CI).
    pub fn mock_text(prompt: &str) -> String {
        format!("mock-generate:{prompt}")
    }
}

impl GenerateBackend for MockBackend {
    fn generate(&self, payload: &GenerateLocalPayload) -> Result<Value, String> {
        payload.validate()?;
        Ok(json!({
            "result": Self::mock_text(&payload.prompt),
            "action": ACTION_GENERATE_LOCAL,
            "backend": MOCK_BACKEND_ID,
        }))
    }
}

/// Phase D activation check. Injected by the plane or tests (CSU ↛ CSU).
///
/// The gate is **activation state**, not a required `model_artifact_ref` on the
/// RFC-0105 payload. Absence of a bound gate is fail-closed.
pub trait ModelActivateGate: Send {
    /// `Ok` if generate may proceed. `Err` becomes [`EventType::CapsuleFailed`].
    fn check_activated(&self, payload: &GenerateLocalPayload) -> Result<(), String>;
}

/// Test double: treat a model as Phase D activated.
#[derive(Debug, Default, Clone, Copy)]
pub struct AlwaysActivated;

impl ModelActivateGate for AlwaysActivated {
    fn check_activated(&self, _payload: &GenerateLocalPayload) -> Result<(), String> {
        Ok(())
    }
}

/// Test double: treat the model as not activated.
#[derive(Debug, Default, Clone, Copy)]
pub struct NeverActivated;

impl ModelActivateGate for NeverActivated {
    fn check_activated(&self, _payload: &GenerateLocalPayload) -> Result<(), String> {
        Err(ACTIVATE_DENIED.into())
    }
}

/// Local LLM execution CSU. Default construction has **no** backend and **no**
/// activate gate (fail-closed).
pub struct ExecutionLlmCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: String,
    backend: Option<Box<dyn GenerateBackend>>,
    activate_gate: Option<Box<dyn ModelActivateGate>>,
}

impl Default for ExecutionLlmCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionLlmCsu {
    /// Construct without a backend. Capsules fail closed until one is bound.
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:execution.llm",
                "execution-llm",
                CsuType::Execution,
                &["CapsuleCreated"],
                &["CapsuleCompleted", "CapsuleFailed"],
            ),
            seq: 1,
            run_nonce: String::from("0"),
            backend: None,
            activate_gate: None,
        }
    }

    /// Namespace ids for multi-run local nodes.
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

    /// Bind [`MockBackend`] (tests / CI / reference plane default).
    pub fn with_mock_backend(self) -> Self {
        self.with_backend(MockBackend)
    }

    /// Bind [`ProcessBackend`] (opt-in local CLI; not the CI default).
    pub fn with_process_backend(self, backend: ProcessBackend) -> Self {
        self.with_backend(backend)
    }

    /// Bind mock unless `AIRA_LLM_BACKEND=process`. Plane/CI must keep mock.
    pub fn with_backend_from_env(self) -> Self {
        if backend_kind_from_env() == PROCESS_BACKEND_ID {
            self.with_process_backend(ProcessBackend::from_env())
        } else {
            self.with_mock_backend()
        }
    }

    /// Bind any [`GenerateBackend`].
    pub fn with_backend(mut self, backend: impl GenerateBackend + 'static) -> Self {
        self.backend = Some(Box::new(backend));
        self
    }

    /// Bind a Phase D activate handle. The plane or tests supply this (CSU ↛ CSU).
    pub fn with_activate_gate(mut self, gate: impl ModelActivateGate + 'static) -> Self {
        self.activate_gate = Some(Box::new(gate));
        self
    }

    fn next_id(&mut self, kind: &str) -> String {
        let id = format!("aira:{kind}:execllm{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }

    /// Parse and validate a generate-local capsule body.
    fn parse_payload(bytes: &[u8]) -> Result<GenerateLocalPayload, String> {
        let value: Value =
            serde_json::from_slice(bytes).map_err(|e| format!("capsule json: {e}"))?;
        let payload: GenerateLocalPayload =
            serde_json::from_value(value).map_err(|e| format!("generate-local payload: {e}"))?;
        payload.validate()?;
        Ok(payload)
    }

    /// Fail closed unless a [`ModelActivateGate`] reports Phase D activation.
    ///
    /// Optional `model_artifact_ref` is accepted by RFC-0105 and forwarded to
    /// the gate; it is **not** required on every payload.
    fn check_activate(&self, payload: &GenerateLocalPayload) -> Result<(), String> {
        match self.activate_gate.as_ref() {
            Some(gate) => gate.check_activated(payload),
            None => Err(ACTIVATE_DENIED.into()),
        }
    }

    fn fail(
        &mut self,
        ctx: &mut CsuExecutionContext<'_, '_>,
        event: &EventDescriptor,
        message: &str,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        let failed = make_event_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &self.next_id("event"),
            EventType::CapsuleFailed,
            event.object_refs.clone(),
            event.artifact_refs.clone(),
            vec![event.event_id.clone()],
            Some(message.into()),
        )
        .map_err(|e| CsuHandlerError {
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

impl Csu for ExecutionLlmCsu {
    fn manifest(&self) -> &CsuManifest {
        &self.manifest
    }

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        if event.event_type != EventType::CapsuleCreated {
            return Ok(vec![]);
        }

        let capsule_id = match event.artifact_refs.first().cloned() {
            Some(id) => id,
            None => return self.fail(ctx, event, "CapsuleCreated missing artifact_ref"),
        };

        let (_desc, bytes) = match ctx.resolve_artifact(&capsule_id) {
            Ok(v) => v,
            Err(e) => {
                return self.fail(ctx, event, &format!("missing capsule artifact: {e}"));
            }
        };

        // Fan-out with execution-basic: skip math/echo/uppercase capsules.
        // A generate-local schema body with the wrong action still fails closed.
        let preview: Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => return self.fail(ctx, event, &format!("capsule json: {e}")),
        };
        let action = preview.get("action").and_then(|v| v.as_str()).unwrap_or("");
        if action != ACTION_GENERATE_LOCAL {
            let schema = preview.get("payload_schema").and_then(|v| v.as_str());
            if schema == Some(PAYLOAD_SCHEMA_ID) {
                return self.fail(
                    ctx,
                    event,
                    &format!("unsupported action: {action} (want {ACTION_GENERATE_LOCAL})"),
                );
            }
            return Ok(vec![]);
        }

        let payload = match Self::parse_payload(&bytes) {
            Ok(p) => p,
            Err(msg) => return self.fail(ctx, event, &msg),
        };

        if let Err(msg) = self.check_activate(&payload) {
            return self.fail(ctx, event, &msg);
        }

        let backend = match self.backend.as_ref() {
            Some(b) => b,
            None => {
                return self.fail(
                    ctx,
                    event,
                    "no generate backend bound (fail-closed; not VERIFIED)",
                );
            }
        };

        match backend.generate(&payload) {
            Ok(result) => {
                let out_payload = json_bytes(&result);
                let out_id = self.next_id("artifact");
                let out_desc = make_artifact_as(
                    self.manifest.csu_id.clone(),
                    self.manifest.publisher_identity.clone(),
                    &out_id,
                    ArtifactType::ExecutionArtifact,
                    &out_payload,
                    vec![event.event_id.clone(), capsule_id.clone()],
                )
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
                ctx.publish_artifact(out_desc.clone(), &out_payload)
                    .map_err(|e| CsuHandlerError {
                        message: e.to_string(),
                    })?;
                let done = make_event_as(
                    self.manifest.csu_id.clone(),
                    self.manifest.publisher_identity.clone(),
                    &self.next_id("event"),
                    EventType::CapsuleCompleted,
                    event.object_refs.clone(),
                    vec![out_desc.artifact_id.clone(), capsule_id],
                    vec![event.event_id.clone()],
                    Some(result.to_string()),
                )
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
                ctx.append_event(done.clone())
                    .map_err(|e| CsuHandlerError {
                        message: e.to_string(),
                    })?;
                Ok(vec![
                    CsuOutput::Artifact {
                        descriptor: out_desc,
                        payload: out_payload,
                    },
                    CsuOutput::Event(done),
                ])
            }
            Err(msg) => self.fail(ctx, event, &msg),
        }
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
    use aira_csu::support::{json_bytes, make_artifact, make_event as mk};
    use aira_event::MemoryEventLog;
    use aira_object::AiraRef;
    use serde_json::json;

    fn valid_generate_body() -> Value {
        json!({
            "payload_schema": PAYLOAD_SCHEMA_ID,
            "action": ACTION_GENERATE_LOCAL,
            "prompt": "Summarize the local Problem Statement without leaving the host.",
            "problem_statement_ref": "aira:problem:01TESTPROBLEM",
            "constraints": { "network": "none", "shell": false },
            "provenance_refs": ["aira:identity:local-test"],
            "signature": {
                "algorithm": "ed25519",
                "key_ref": "aira:identity:local-test",
                "signature_value": "TESTSIG"
            }
        })
    }

    fn bind_capsule(store: &mut CasArtifactStore, body: &Value) -> AiraRef {
        let payload = json_bytes(body);
        let desc = make_artifact(
            "aira:artifact:gencap1",
            ArtifactType::CustomArtifact,
            &payload,
            vec![],
        );
        let id = desc.artifact_id.clone();
        store.publish(desc, &payload).unwrap();
        id
    }

    fn created_event(cap: AiraRef) -> EventDescriptor {
        mk(
            "aira:event:gencap1",
            EventType::CapsuleCreated,
            vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            vec![cap],
            vec![],
            None,
        )
    }

    fn completed(outs: &[CsuOutput]) -> bool {
        outs.iter().any(|o| {
            matches!(
                o,
                CsuOutput::Event(e) if e.event_type == EventType::CapsuleCompleted
            )
        })
    }

    fn failed(outs: &[CsuOutput]) -> bool {
        outs.iter().any(|o| {
            matches!(
                o,
                CsuOutput::Event(e) if e.event_type == EventType::CapsuleFailed
            )
        })
    }

    fn has_verified_result(outs: &[CsuOutput]) -> bool {
        outs.iter().any(|o| {
            matches!(
                o,
                CsuOutput::Artifact { descriptor, .. }
                    if descriptor.artifact_type == ArtifactType::VerifiedResultArtifact
            )
        })
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_is_execution_type() {
        let csu = ExecutionLlmCsu::new();
        assert_eq!(csu.manifest().csu_type, CsuType::Execution);
        assert_eq!(csu.manifest().csu_id.as_str(), "aira:csu:execution.llm");
        csu.manifest().validate_for_registration().unwrap();
    }

    #[test]
    fn mock_backend_completes_valid_generate_local() {
        let mut csu = ExecutionLlmCsu::new()
            .with_mock_backend()
            .with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let body = valid_generate_body();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate(PAYLOAD_SCHEMA_ID, &body).unwrap();
        let cap = bind_capsule(&mut store, &body);
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(completed(&outs), "expected CapsuleCompleted: {outs:?}");
        assert!(!failed(&outs));
        assert!(!has_verified_result(&outs));
        let result = outs
            .iter()
            .find_map(|o| match o {
                CsuOutput::Artifact {
                    payload,
                    descriptor,
                } => {
                    assert_eq!(descriptor.artifact_type, ArtifactType::ExecutionArtifact);
                    Some(serde_json::from_slice::<Value>(payload).unwrap())
                }
                _ => None,
            })
            .expect("execution artifact");
        assert_eq!(
            result["result"],
            json!(MockBackend::mock_text(
                "Summarize the local Problem Statement without leaving the host."
            ))
        );
        assert_eq!(result["backend"], json!(MOCK_BACKEND_ID));
        assert_eq!(result["action"], json!(ACTION_GENERATE_LOCAL));
    }

    #[test]
    fn missing_backend_is_capsule_failed() {
        let mut csu = ExecutionLlmCsu::new().with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, &valid_generate_body());
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs), "expected CapsuleFailed: {outs:?}");
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message } if message.contains("no generate backend bound")
        )));
    }

    #[test]
    fn inactive_model_is_capsule_failed() {
        let mut csu = ExecutionLlmCsu::new().with_mock_backend();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, &valid_generate_body());
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs), "expected CapsuleFailed: {outs:?}");
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message } if message.contains(ACTIVATE_DENIED)
        )));
    }

    #[test]
    fn never_activated_gate_is_capsule_failed() {
        let mut csu = ExecutionLlmCsu::new()
            .with_mock_backend()
            .with_activate_gate(NeverActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, &valid_generate_body());
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs), "expected CapsuleFailed: {outs:?}");
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message } if message.contains(ACTIVATE_DENIED)
        )));
    }

    #[test]
    fn activated_mock_completes_without_model_artifact_ref() {
        let mut csu = ExecutionLlmCsu::new()
            .with_mock_backend()
            .with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut body = valid_generate_body();
        body.as_object_mut().unwrap().remove("model_artifact_ref");
        let cap = bind_capsule(&mut store, &body);
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(completed(&outs), "expected CapsuleCompleted: {outs:?}");
        assert!(!failed(&outs));
        assert!(!has_verified_result(&outs));
    }

    #[test]
    fn wrong_action_is_capsule_failed() {
        let mut csu = ExecutionLlmCsu::new().with_mock_backend();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut body = valid_generate_body();
        body.as_object_mut()
            .unwrap()
            .insert("action".into(), json!("math.eval.safe"));
        let cap = bind_capsule(&mut store, &body);
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs));
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
    }

    #[test]
    fn network_not_none_is_capsule_failed_aira_mediated() {
        let mut csu = ExecutionLlmCsu::new()
            .with_mock_backend()
            .with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut body = valid_generate_body();
        body["constraints"]["network"] = json!("full");
        let cap = bind_capsule(&mut store, &body);
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs));
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message }
                if message.contains(NETWORK_NONE_CONTRACT)
        )));
    }

    #[test]
    fn extra_properties_fail_closed() {
        let mut csu = ExecutionLlmCsu::new().with_mock_backend();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut body = valid_generate_body();
        body.as_object_mut()
            .unwrap()
            .insert("gpu_id".into(), json!("0"));
        let cap = bind_capsule(&mut store, &body);
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs));
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
    }

    #[test]
    fn missing_prompt_is_capsule_failed() {
        let mut csu = ExecutionLlmCsu::new().with_mock_backend();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut body = valid_generate_body();
        body.as_object_mut().unwrap().remove("prompt");
        let cap = bind_capsule(&mut store, &body);
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs));
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
    }

    #[test]
    fn math_eval_capsule_is_skipped_for_plane_fan_out() {
        let mut csu = ExecutionLlmCsu::new().with_mock_backend();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let body = json!({
            "action": "math.eval.safe",
            "expression": "2+2",
            "constraints": { "network": "none", "shell": false }
        });
        let cap = bind_capsule(&mut store, &body);
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(outs.is_empty(), "must not fail C1 capsules: {outs:?}");
    }

    #[test]
    fn missing_process_binary_is_capsule_failed() {
        let mut csu = ExecutionLlmCsu::new()
            .with_process_backend(ProcessBackend::new(
                "aira-llm-process-missing-bin-215-do-not-install",
            ))
            .with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, &valid_generate_body());
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs), "expected CapsuleFailed: {outs:?}");
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message } if message.contains(MISSING_BINARY)
        )));
    }

    #[test]
    fn missing_process_binary_does_not_skip_activate_gate() {
        let mut csu = ExecutionLlmCsu::new().with_process_backend(ProcessBackend::new(
            "aira-llm-process-missing-bin-215-do-not-install",
        ));
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, &valid_generate_body());
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs));
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message } if message.contains(ACTIVATE_DENIED)
        )));
    }

    #[test]
    fn backend_from_env_defaults_to_mock_not_process() {
        assert_eq!(backend_kind_from(None), MOCK_BACKEND_ID);
        assert_eq!(backend_kind_from(Some("mock")), MOCK_BACKEND_ID);
        assert_ne!(backend_kind_from(None), PROCESS_BACKEND_ID);
    }

    #[cfg(unix)]
    #[test]
    fn nonzero_process_is_capsule_failed() {
        let mut csu = ExecutionLlmCsu::new()
            .with_process_backend(ProcessBackend::new("/bin/false"))
            .with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, &valid_generate_body());
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs), "expected CapsuleFailed: {outs:?}");
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message } if message.contains(NONZERO_EXIT)
        )));
    }

    #[cfg(unix)]
    #[test]
    fn echo_process_backend_completes_without_ollama() {
        let mut csu = ExecutionLlmCsu::new()
            .with_process_backend(ProcessBackend::new("/bin/echo"))
            .with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, &valid_generate_body());
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(completed(&outs), "expected CapsuleCompleted: {outs:?}");
        assert!(!failed(&outs));
        assert!(!has_verified_result(&outs));
        let result = outs
            .iter()
            .find_map(|o| match o {
                CsuOutput::Artifact { payload, .. } => {
                    Some(serde_json::from_slice::<Value>(payload).unwrap())
                }
                _ => None,
            })
            .expect("execution artifact");
        assert_eq!(result["backend"], json!(PROCESS_BACKEND_ID));
        assert_eq!(
            result["result"],
            json!("Summarize the local Problem Statement without leaving the host.")
        );
    }

    #[cfg(unix)]
    #[test]
    fn process_timeout_is_capsule_failed() {
        let mut csu = ExecutionLlmCsu::new()
            .with_process_backend(
                ProcessBackend::new("/bin/sleep")
                    .with_timeout(std::time::Duration::from_millis(80)),
            )
            .with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut body = valid_generate_body();
        body.as_object_mut()
            .unwrap()
            .insert("prompt".into(), json!("2"));
        let cap = bind_capsule(&mut store, &body);
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs), "expected CapsuleFailed: {outs:?}");
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message } if message.contains(TIMED_OUT)
        )));
    }

    #[cfg(unix)]
    #[test]
    fn stdout_overflow_is_capsule_failed() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("overflow-stdout");
        let kb = (PIPE_STDOUT_LIMIT / 1024) + 512;
        std::fs::write(
            &script,
            format!("#!/bin/sh\ndd if=/dev/zero bs=1024 count={kb} 2>/dev/null\n"),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&script).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&script, perm).unwrap();
        let mut csu = ExecutionLlmCsu::new()
            .with_process_backend(
                ProcessBackend::new(&script).with_timeout(std::time::Duration::from_secs(10)),
            )
            .with_activate_gate(AlwaysActivated);
        let mut log = MemoryEventLog::new();
        let mut store = CasArtifactStore::open(dir.path().join("arts")).unwrap();
        let cap = bind_capsule(&mut store, &valid_generate_body());
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let outs = csu.on_event(&created_event(cap), &mut ctx).unwrap();
        assert!(failed(&outs), "expected CapsuleFailed: {outs:?}");
        assert!(!completed(&outs));
        assert!(!has_verified_result(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Failure { message } if message.contains(PIPE_OVERFLOW)
        )));
    }
}
