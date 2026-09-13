//! Reduction-basic CSU (Issue #42).
//!
//! Prefers Ready Solution / Knowledge reuse; otherwise Negative Lookup + Execution Capsule.
//!
//! QUEUE `#212`: catalog bind is by action/capability **string**. Echo and uppercase keep
//! their existing binds. `Calculate 2 + 2` stays [`ACTION_MATH_EVAL_SAFE`]. Any other
//! non-math statement binds [`ACTION_GENERATE_LOCAL`] (RFC-0105 payload). This crate does
//! **not** import execution CSUs (CSU ↛ CSU). Plane dispatch of generate is `#213` (RFC-0108).

use aira_artifact::ArtifactType;
use aira_csu::support::{
    basic_manifest, json_bytes, local_signature_over, make_artifact_as, make_event_as,
};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::AiraRef;
use serde_json::{json, Map, Value};

/// Safe arithmetic action (C1 `Calculate 2 + 2` / `c1.pipeline.calculate_2_plus_2`).
pub const ACTION_MATH_EVAL_SAFE: &str = "math.eval.safe";
/// Existing echo catalog entry.
pub const ACTION_TEXT_ECHO: &str = "text.echo";
/// Existing uppercase catalog entry.
pub const ACTION_TEXT_UPPERCASE: &str = "text.uppercase";
/// Host-local generate action (RFC-0105). Selected here; executed by execution-llm in `#213`.
pub const ACTION_GENERATE_LOCAL: &str = "text.generate.local";
/// Payload `$id` for generate-local CustomArtifact content.
pub const PAYLOAD_SCHEMA_GENERATE_LOCAL: &str = "aira:schema:execution:generate-local:0.1";

/// Bind a Problem Statement to a catalog action without importing execution CSUs.
pub fn catalog_action(statement: &str) -> &'static str {
    let lower = statement.to_lowercase();
    if lower.contains("echo") {
        ACTION_TEXT_ECHO
    } else if lower.contains("upper") {
        ACTION_TEXT_UPPERCASE
    } else if is_math_eval_safe(statement) {
        ACTION_MATH_EVAL_SAFE
    } else {
        ACTION_GENERATE_LOCAL
    }
}

/// True for C1-style `Calculate …` with a digit, or a bare arithmetic expression.
fn is_math_eval_safe(statement: &str) -> bool {
    let lower = statement.to_lowercase();
    let has_digit = statement.chars().any(|c| c.is_ascii_digit());
    if lower.contains("calculate") && has_digit {
        return true;
    }
    let cleaned: String = statement.chars().filter(|c| !c.is_whitespace()).collect();
    is_pure_math_expression(&cleaned)
}

/// Public classifier for admission constraint gating (`#334` / RFC-0218).
///
/// Deterministic math (`math.eval.safe`) cannot honor model/placement LLM
/// constraints — callers must reject those as unsupported.
pub fn problem_binds_math_eval_safe(statement: &str) -> bool {
    is_math_eval_safe(statement)
}

fn is_pure_math_expression(cleaned: &str) -> bool {
    !cleaned.is_empty()
        && cleaned.chars().any(|c| c.is_ascii_digit())
        && cleaned
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.'))
}

fn is_math_token(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.'))
        && token
            .chars()
            .any(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')'))
}

/// Extract the arithmetic expression that must appear on the math capsule (`#332` / RFC-0216).
///
/// Never invents a default `2+2`. Unsupported / empty extracts fail closed.
fn extract_math_expression(statement: &str) -> Result<String, String> {
    let trimmed = statement.trim();
    if trimmed.is_empty() {
        return Err("empty math statement".into());
    }

    let after_calculate = {
        let lower = trimmed.to_lowercase();
        if lower.starts_with("calculate") {
            trimmed["calculate".len()..].trim()
        } else {
            trimmed
        }
    };

    let cleaned: String = after_calculate
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if is_pure_math_expression(&cleaned) {
        return Ok(cleaned);
    }

    let joined: String = trimmed
        .split_whitespace()
        .filter(|t| is_math_token(t))
        .collect();
    if is_pure_math_expression(&joined) {
        return Ok(joined);
    }

    Err(format!(
        "unsupported math expression in statement (refusing default 2+2): {statement}"
    ))
}

/// RFC-0105 generate-local payload (CustomArtifact content). Extra capsule fields would
/// fail `additionalProperties: false` / execution-llm `deny_unknown_fields` in `#213`.
///
/// `#328`: optional `model_artifact_ref` from admission snapshot when present.
fn generate_local_payload(
    problem_ref: &AiraRef,
    prompt: &str,
    provenance: &AiraRef,
    model_ref: Option<&str>,
) -> Value {
    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!(PAYLOAD_SCHEMA_GENERATE_LOCAL),
    );
    body.insert("action".into(), json!(ACTION_GENERATE_LOCAL));
    body.insert("prompt".into(), json!(prompt));
    body.insert("problem_statement_ref".into(), json!(problem_ref.as_str()));
    if let Some(m) = model_ref.filter(|s| !s.is_empty()) {
        body.insert("model_artifact_ref".into(), json!(m));
    }
    body.insert(
        "constraints".into(),
        json!({ "network": "none", "shell": false }),
    );
    body.insert("provenance_refs".into(), json!([provenance.as_str()]));
    let for_sign = Value::Object(body.clone());
    let bytes = serde_json::to_vec(&for_sign).expect("generate-local sign body");
    let sig = local_signature_over(&bytes);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).expect("signature json"),
    );
    Value::Object(body)
}

/// Read optional admitted `model_ref` from ContextResolved artifact (`#328`).
fn admission_model_ref_from_context(
    ctx: &mut CsuExecutionContext<'_, '_>,
    context_ref: &AiraRef,
) -> Option<String> {
    let (_, bytes) = ctx.resolve_artifact(context_ref).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    v.pointer("/resolved_factors/admission_snapshot/model_ref")
        .and_then(|m| m.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Local reduction / reuse CSU.
pub struct ReductionBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: String,
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
            run_nonce: String::from("0"),
            ready_solutions: vec![],
            knowledge: vec![],
        }
    }

    pub fn with_ready_solution(mut self, id: AiraRef) -> Self {
        self.ready_solutions.push(id);
        self
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
        let action = catalog_action(&statement);
        let (capsule, artifact_type) = if action == ACTION_GENERATE_LOCAL {
            let prompt = if statement.is_empty() {
                problem_ref.as_str().to_string()
            } else {
                statement.clone()
            };
            let model_ref = admission_model_ref_from_context(ctx, &context_ref);
            (
                generate_local_payload(
                    &problem_ref,
                    &prompt,
                    &event.event_id,
                    model_ref.as_deref(),
                ),
                ArtifactType::CustomArtifact,
            )
        } else if action == ACTION_MATH_EVAL_SAFE {
            let expr = match extract_math_expression(&statement) {
                Ok(e) => e,
                Err(msg) => {
                    let failed = make_event_as(
                        self.manifest.csu_id.clone(),
                        self.manifest.publisher_identity.clone(),
                        &self.next_id("event"),
                        EventType::CapsuleFailed,
                        vec![problem_ref.clone()],
                        vec![neg_desc.artifact_id.clone()],
                        vec![event.event_id.clone()],
                        Some(msg.clone()),
                    )
                    .map_err(|e| CsuHandlerError {
                        message: e.to_string(),
                    })?;
                    ctx.append_event(failed.clone())
                        .map_err(|e| CsuHandlerError {
                            message: e.to_string(),
                        })?;
                    outs.push(CsuOutput::Failure { message: msg });
                    outs.push(CsuOutput::Event(failed));
                    return Ok(outs);
                }
            };
            (
                json!({
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
                }),
                ArtifactType::ExecutionArtifact,
            )
        } else {
            (
                json!({
                    "capsule_id": format!("aira:capsule:red{}", self.seq),
                    "problem_statement_ref": problem_ref.as_str(),
                    "context_ref": context_ref.as_str(),
                    "action": action,
                    "expression": statement.clone(),
                    "required_capabilities": [action],
                    "input_artifact_refs": [context_ref.as_str()],
                    "constraints": { "network": "none", "shell": false },
                    "policy_refs": ["aira:policy:default"],
                    "provenance_refs": [event.event_id.as_str()]
                }),
                ArtifactType::ExecutionArtifact,
            )
        };
        let cap_payload = json_bytes(&capsule);
        let cap_id = self.next_id("artifact");
        let cap_desc = make_artifact_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &cap_id,
            artifact_type,
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
    use aira_artifact::{ArtifactStore, CasArtifactStore};
    use aira_csu::support::make_event as mk;
    use aira_event::MemoryEventLog;

    fn reduce(statement: &str) -> Vec<CsuOutput> {
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
            Some(statement.into()),
        );
        csu.on_event(&ev, &mut ctx).unwrap()
    }

    fn capsule_action(outs: &[CsuOutput]) -> String {
        outs.iter()
            .find_map(|o| match o {
                CsuOutput::Event(e) if e.event_type == EventType::CapsuleCreated => {
                    e.payload_ref.clone()
                }
                _ => None,
            })
            .expect("CapsuleCreated")
    }

    fn capsule_json(outs: &[CsuOutput]) -> (ArtifactType, Value) {
        outs.iter()
            .find_map(|o| match o {
                CsuOutput::Artifact {
                    descriptor,
                    payload,
                } if descriptor.artifact_type == ArtifactType::ExecutionArtifact
                    || descriptor.artifact_type == ArtifactType::CustomArtifact =>
                {
                    Some((
                        descriptor.artifact_type,
                        serde_json::from_slice(payload).unwrap(),
                    ))
                }
                _ => None,
            })
            .expect("capsule artifact")
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn catalog_action_splits_math_echo_upper_and_generate() {
        assert_eq!(catalog_action("Calculate 2 + 2"), ACTION_MATH_EVAL_SAFE);
        assert_eq!(catalog_action("2+2"), ACTION_MATH_EVAL_SAFE);
        assert_eq!(catalog_action("echo hello"), ACTION_TEXT_ECHO);
        assert_eq!(catalog_action("uppercase foo"), ACTION_TEXT_UPPERCASE);
        assert_eq!(
            catalog_action("Summarize the local Problem Statement without leaving the host."),
            ACTION_GENERATE_LOCAL
        );
    }

    #[test]
    fn creates_negative_lookup_and_capsule_when_no_reuse() {
        let outs = reduce("Calculate 2 + 2");
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
        assert_eq!(capsule_action(&outs), ACTION_MATH_EVAL_SAFE);
    }

    #[test]
    fn calculate_2_plus_2_binds_math_eval_safe() {
        let outs = reduce("Calculate 2 + 2");
        assert_eq!(capsule_action(&outs), ACTION_MATH_EVAL_SAFE);
        let (ty, body) = capsule_json(&outs);
        assert_eq!(ty, ArtifactType::ExecutionArtifact);
        assert_eq!(body["action"], json!(ACTION_MATH_EVAL_SAFE));
        assert_eq!(body["expression"], json!("2+2"));
        assert_eq!(body["constraints"]["network"], json!("none"));
        assert_eq!(body["constraints"]["shell"], json!(false));
        assert_ne!(body["action"], json!(ACTION_GENERATE_LOCAL));
    }

    #[test]
    fn math_capsule_preserves_sub_div_and_bare_number() {
        for (statement, expr) in [
            ("9-3", "9-3"),
            ("9/3", "9/3"),
            ("42", "42"),
            ("Calculate 9 - 3", "9-3"),
            ("Calculate 9 / 3", "9/3"),
            ("Calculate 42", "42"),
        ] {
            let outs = reduce(statement);
            assert_eq!(
                capsule_action(&outs),
                ACTION_MATH_EVAL_SAFE,
                "action for {statement}"
            );
            let (_, body) = capsule_json(&outs);
            assert_eq!(
                body["expression"],
                json!(expr),
                "expression for {statement}"
            );
        }
    }

    #[test]
    fn unsupported_math_statement_fails_without_default_two_plus_two() {
        let outs = reduce("Calculate xyz 7q");
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::CapsuleFailed
        )));
        assert!(outs.iter().any(|o| matches!(o, CsuOutput::Failure { .. })));
        assert!(
            outs.iter().all(|o| !matches!(
                o,
                CsuOutput::Event(e) if e.event_type == EventType::CapsuleCreated
            )),
            "must not CapsuleCreated with invented 2+2"
        );
        assert!(
            !outs.iter().any(|o| match o {
                CsuOutput::Artifact { payload, .. } => {
                    let v: Value = serde_json::from_slice(payload).unwrap_or(json!({}));
                    v.get("expression") == Some(&json!("2+2"))
                }
                _ => false,
            }),
            "must not publish expression 2+2"
        );
    }

    #[test]
    fn extract_math_expression_helpers() {
        assert_eq!(extract_math_expression("2+2").unwrap(), "2+2");
        assert_eq!(extract_math_expression("9-3").unwrap(), "9-3");
        assert_eq!(extract_math_expression("Calculate 42").unwrap(), "42");
        assert!(extract_math_expression("Calculate only words").is_err());
    }

    #[test]
    fn non_math_prompt_binds_generate_local() {
        let prompt = "Summarize the local Problem Statement without leaving the host.";
        let outs = reduce(prompt);
        assert_eq!(capsule_action(&outs), ACTION_GENERATE_LOCAL);
        let (ty, body) = capsule_json(&outs);
        assert_eq!(ty, ArtifactType::CustomArtifact);
        assert_eq!(body["payload_schema"], json!(PAYLOAD_SCHEMA_GENERATE_LOCAL));
        assert_eq!(body["action"], json!(ACTION_GENERATE_LOCAL));
        assert_eq!(body["prompt"], json!(prompt));
        assert_eq!(body["constraints"]["network"], json!("none"));
        assert_eq!(body["constraints"]["shell"], json!(false));
        assert!(body.get("signature").is_some());
        assert!(body.get("gpu_id").is_none());
        assert!(body.get("expression").is_none());
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate(PAYLOAD_SCHEMA_GENERATE_LOCAL, &body).unwrap();
    }

    #[test]
    fn generate_local_capsule_carries_admission_model_ref() {
        let mut csu = ReductionBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let ctx_body = json!({
            "resolved_factors": {
                "admission_snapshot": {
                    "kind": "admission_snapshot",
                    "model_ref": "aira:model:chosen-x",
                    "statement_content_hash": "sha256:deadbeef"
                }
            }
        });
        let ctx_payload = json_bytes(&ctx_body);
        let ctx_desc = aira_csu::support::make_artifact(
            "aira:artifact:ctx-admit",
            ArtifactType::OperationalArtifact,
            &ctx_payload,
            vec![],
        );
        let ctx_id = ctx_desc.artifact_id.clone();
        store.publish(ctx_desc, &ctx_payload).unwrap();
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let prompt = "Summarize the local Problem Statement without leaving the host.";
        let ev = mk(
            "aira:event:c-admit",
            EventType::ContextResolved,
            vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            vec![ctx_id],
            vec![],
            Some(prompt.into()),
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        let (_, body) = capsule_json(&outs);
        assert_eq!(body["action"], json!(ACTION_GENERATE_LOCAL));
        assert_eq!(body["model_artifact_ref"], json!("aira:model:chosen-x"));
    }

    #[test]
    fn echo_and_uppercase_keep_existing_binds() {
        let echo = reduce("echo hello");
        assert_eq!(capsule_action(&echo), ACTION_TEXT_ECHO);
        let (ty, body) = capsule_json(&echo);
        assert_eq!(ty, ArtifactType::ExecutionArtifact);
        assert_eq!(body["action"], json!(ACTION_TEXT_ECHO));

        let upper = reduce("uppercase foo");
        assert_eq!(capsule_action(&upper), ACTION_TEXT_UPPERCASE);
        let (ty, body) = capsule_json(&upper);
        assert_eq!(ty, ArtifactType::ExecutionArtifact);
        assert_eq!(body["action"], json!(ACTION_TEXT_UPPERCASE));
    }
}
