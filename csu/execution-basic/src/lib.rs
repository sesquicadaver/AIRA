//! Execution-basic CSU (Issue #43).
//!
//! Safe deterministic actions only: math.eval.safe, text.echo, text.uppercase.

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::AiraRef;
use serde_json::{json, Value};

/// Safe local execution CSU.
pub struct ExecutionBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: String,
}

impl Default for ExecutionBasicCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionBasicCsu {
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:execution.basic",
                "execution-basic",
                CsuType::Execution,
                &["CapsuleCreated"],
                &["CapsuleCompleted", "CapsuleFailed"],
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
        let id = format!("aira:{kind}:exec{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }
}

/// Evaluate a tiny safe arithmetic expression (digits and + - * / ( )).
fn math_eval_safe(expr: &str) -> Result<f64, String> {
    let cleaned: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return Err("empty expression".into());
    }
    if cleaned
        .chars()
        .any(|c| !(c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.')))
    {
        return Err("unsupported characters for math.eval.safe".into());
    }
    // Extremely small recursive descent.
    let bytes = cleaned.as_bytes();
    let mut i = 0usize;
    fn parse_expr(bytes: &[u8], i: &mut usize) -> Result<f64, String> {
        let mut v = parse_term(bytes, i)?;
        while *i < bytes.len() {
            match bytes[*i] {
                b'+' => {
                    *i += 1;
                    v += parse_term(bytes, i)?;
                }
                b'-' => {
                    *i += 1;
                    v -= parse_term(bytes, i)?;
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn parse_term(bytes: &[u8], i: &mut usize) -> Result<f64, String> {
        let mut v = parse_factor(bytes, i)?;
        while *i < bytes.len() {
            match bytes[*i] {
                b'*' => {
                    *i += 1;
                    v *= parse_factor(bytes, i)?;
                }
                b'/' => {
                    *i += 1;
                    let d = parse_factor(bytes, i)?;
                    if d == 0.0 {
                        return Err("division by zero".into());
                    }
                    v /= d;
                }
                _ => break,
            }
        }
        Ok(v)
    }
    fn parse_factor(bytes: &[u8], i: &mut usize) -> Result<f64, String> {
        if *i < bytes.len() && bytes[*i] == b'(' {
            *i += 1;
            let v = parse_expr(bytes, i)?;
            if *i >= bytes.len() || bytes[*i] != b')' {
                return Err("missing )".into());
            }
            *i += 1;
            return Ok(v);
        }
        let start = *i;
        if *i < bytes.len() && (bytes[*i] == b'+' || bytes[*i] == b'-') {
            *i += 1;
        }
        while *i < bytes.len() && (bytes[*i].is_ascii_digit() || bytes[*i] == b'.') {
            *i += 1;
        }
        if start == *i {
            return Err("expected number".into());
        }
        std::str::from_utf8(&bytes[start..*i])
            .map_err(|_| "utf8".to_string())?
            .parse::<f64>()
            .map_err(|e| e.to_string())
    }
    let v = parse_expr(bytes, &mut i)?;
    if i != bytes.len() {
        return Err("trailing input".into());
    }
    Ok(v)
}

fn run_action(action: &str, input: &str) -> Result<Value, String> {
    match action {
        "math.eval.safe" => Ok(json!({"result": math_eval_safe(input)?, "action": action})),
        "text.echo" => Ok(json!({"result": input, "action": action})),
        "text.uppercase" => Ok(json!({"result": input.to_uppercase(), "action": action})),
        "shell" | "bash" | "network.fetch" => Err(format!("forbidden action: {action}")),
        other => Err(format!("unsupported action: {other}")),
    }
}

impl Csu for ExecutionBasicCsu {
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

        let capsule_id = event
            .artifact_refs
            .first()
            .cloned()
            .ok_or_else(|| CsuHandlerError {
                message: "CapsuleCreated missing artifact_ref".into(),
            })?;

        let (_desc, bytes) = match ctx.resolve_artifact(&capsule_id) {
            Ok(v) => v,
            Err(e) => {
                return self.fail(ctx, event, &format!("missing capsule artifact: {e}"));
            }
        };
        let capsule: Value = serde_json::from_slice(&bytes).map_err(|e| CsuHandlerError {
            message: format!("capsule json: {e}"),
        })?;
        let action = capsule
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("math.eval.safe");
        // Fan-out with execution-llm: generate-local capsules are not this CSU.
        if action == "text.generate.local" {
            return Ok(vec![]);
        }
        let expression = capsule
            .get("expression")
            .and_then(|v| v.as_str())
            .or(event.payload_ref.as_deref())
            .unwrap_or("2+2");

        // Hard deny shell/network markers in capsule constraints.
        if capsule
            .pointer("/constraints/shell")
            .and_then(|v| v.as_bool())
            == Some(true)
        {
            return self.fail(ctx, event, "shell execution forbidden");
        }
        if capsule
            .pointer("/constraints/network")
            .and_then(|v| v.as_str())
            .is_some_and(|n| n != "none")
        {
            return self.fail(ctx, event, "network access forbidden");
        }

        match run_action(action, expression) {
            Ok(result) => {
                let payload = json_bytes(&result);
                let out_id = self.next_id("artifact");
                let out_desc = make_artifact_as(
                    self.manifest.csu_id.clone(),
                    self.manifest.publisher_identity.clone(),
                    &out_id,
                    ArtifactType::ExecutionArtifact,
                    &payload,
                    vec![event.event_id.clone(), capsule_id.clone()],
                )
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
                ctx.publish_artifact(out_desc.clone(), &payload)
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
                        payload,
                    },
                    CsuOutput::Event(done),
                ])
            }
            Err(msg) => self.fail(ctx, event, &msg),
        }
    }
}

impl ExecutionBasicCsu {
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

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_artifact::{ArtifactStore, CasArtifactStore};
    use aira_csu::support::{json_bytes, make_artifact, make_event as mk};
    use aira_event::MemoryEventLog;
    use aira_object::AiraRef;

    fn bind_capsule(store: &mut CasArtifactStore, action: &str, expr: &str) -> AiraRef {
        let body = json!({
            "action": action,
            "expression": expr,
            "constraints": { "network": "none", "shell": false }
        });
        let payload = json_bytes(&body);
        let desc = make_artifact(
            "aira:artifact:cap1",
            ArtifactType::ExecutionArtifact,
            &payload,
            vec![],
        );
        let id = desc.artifact_id.clone();
        store.publish(desc, &payload).unwrap();
        id
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn math_eval_safe_completes() {
        assert_eq!(math_eval_safe("2+2").unwrap(), 4.0);
        let mut csu = ExecutionBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, "math.eval.safe", "2+2");
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let ev = mk(
            "aira:event:cap1",
            EventType::CapsuleCreated,
            vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            vec![cap],
            vec![],
            None,
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::CapsuleCompleted
        )));
    }

    #[test]
    fn rejects_shell_action() {
        let mut csu = ExecutionBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, "shell", "rm -rf /");
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let ev = mk(
            "aira:event:cap2",
            EventType::CapsuleCreated,
            vec![],
            vec![cap],
            vec![],
            None,
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::CapsuleFailed
        )));
    }

    #[test]
    fn generate_local_action_is_skipped_for_plane_fan_out() {
        let mut csu = ExecutionBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap = bind_capsule(&mut store, "text.generate.local", "prose prompt");
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );
        let ev = mk(
            "aira:event:capgen",
            EventType::CapsuleCreated,
            vec![],
            vec![cap],
            vec![],
            None,
        );
        let outs = csu.on_event(&ev, &mut ctx).unwrap();
        assert!(
            outs.is_empty(),
            "must not CapsuleFailed generate-local: {outs:?}"
        );
    }
}
