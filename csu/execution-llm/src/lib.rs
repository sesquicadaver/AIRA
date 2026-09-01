//! Execution-llm CSU (QUEUE #211 / Analyze-246; plane register `#213`).
//!
//! Host-local `text.generate.local` capsules complete only through a bound
//! [`GenerateBackend`]. [`MockBackend`] is deterministic and never shells out
//! or uses the network. Missing backend or invalid payload → [`EventType::CapsuleFailed`],
//! never a fake VERIFIED result.
//!
//! OperationalPlane registers this CSU with [`MockBackend`] (`#213`). Capsules
//! whose action is not generate-local are skipped so fan-out with
//! execution-basic does not fail C1 `math.eval.safe`. Activate policy is `#214`.
//! Process/ollama backend is `#215`. No Cargo dep on inventory/acquisition CSUs.

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

/// Constraints frozen by the generate-local schema (`network=none`, `shell=false`).
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
            return Err("network access forbidden".into());
        }
        if self.constraints.shell {
            return Err("shell execution forbidden".into());
        }
        Ok(())
    }
}

/// Local generate backend. Implementations must not shell out or use the network.
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

/// Local LLM execution CSU. Default construction has **no** backend (fail-closed).
pub struct ExecutionLlmCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: String,
    backend: Option<Box<dyn GenerateBackend>>,
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

    /// Bind [`MockBackend`] (tests / CI / reference plane until `#215`).
    pub fn with_mock_backend(self) -> Self {
        self.with_backend(MockBackend)
    }

    /// Bind any [`GenerateBackend`]. Process/CLI adapters belong in `#215`.
    pub fn with_backend(mut self, backend: impl GenerateBackend + 'static) -> Self {
        self.backend = Some(Box::new(backend));
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

    /// `#214` activate gate lives here as a fail-closed hook only.
    ///
    /// This atom does **not** implement Phase D activate policy. Optional
    /// `model_artifact_ref` is accepted by the schema and ignored for gating.
    fn activate_gate_placeholder(_payload: &GenerateLocalPayload) -> Result<(), String> {
        // TODO(#214): generate without Phase D activate → CapsuleFailed + Evidence, not VERIFIED.
        Ok(())
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

        if let Err(msg) = Self::activate_gate_placeholder(&payload) {
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
        let mut csu = ExecutionLlmCsu::new().with_mock_backend();
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
        let mut csu = ExecutionLlmCsu::new();
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
}
