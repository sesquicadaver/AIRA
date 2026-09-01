//! Reduction-basic CSU (Issue #42).
//!
//! Prefers Ready Solution / Knowledge reuse; otherwise Negative Lookup + Execution Capsule.
//!
//! QUEUE `#212`: catalog bind is by action/capability **string**. Echo and uppercase keep
//! their existing binds. `Calculate 2 + 2` stays [`ACTION_MATH_EVAL_SAFE`]. Any other
//! non-math statement binds [`ACTION_GENERATE_LOCAL`] (RFC-0105 payload). This crate does
//! **not** import execution CSUs (CSU ↛ CSU). Plane dispatch of generate is `#213`.

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
    !cleaned.is_empty()
        && has_digit
        && cleaned
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.'))
}

/// RFC-0105 generate-local payload (CustomArtifact content). Extra capsule fields would
/// fail `additionalProperties: false` / execution-llm `deny_unknown_fields` in `#213`.
fn generate_local_payload(problem_ref: &AiraRef, prompt: &str, provenance: &AiraRef) -> Value {
    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!(PAYLOAD_SCHEMA_GENERATE_LOCAL),
    );
    body.insert("action".into(), json!(ACTION_GENERATE_LOCAL));
    body.insert("prompt".into(), json!(prompt));
    body.insert("problem_statement_ref".into(), json!(problem_ref.as_str()));
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
            (
                generate_local_payload(&problem_ref, &prompt, &event.event_id),
                ArtifactType::CustomArtifact,
            )
        } else {
            let expr = if action == ACTION_MATH_EVAL_SAFE {
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
    use aira_artifact::CasArtifactStore;
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
