//! Verification-basic CSU (Issue #44).
//!
//! Distinguishes Output Artifact from Verified Result Artifact.
//! `#187`: `math.eval.safe` is independently evaluated; a wrong finite result is not VERIFIED.
//! `#205`: `text.echo` / `text.uppercase` compare claimed `result` to capsule `expression`.
//! Phase U `#314`: action/expression are sourced from the admitted capsule
//! (`CapsuleCompleted` `artifact_refs[1]`), not from executor output. A substituted
//! but internally consistent output must not VERIFIED.

use aira_artifact::ArtifactType;
use aira_csu::support::{
    basic_manifest, json_bytes, make_artifact_as, make_event_as, mvp_timestamp,
};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::{canonical_json_bytes, signature_for_tenant, AiraRef, ContentHash};
use serde_json::{json, Value};

/// Deterministic verification CSU.
pub struct VerificationBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: String,
}

impl Default for VerificationBasicCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationBasicCsu {
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:verification.basic",
                "verification-basic",
                CsuType::Verification,
                &["CapsuleCompleted"],
                &[
                    "VerificationCompleted",
                    "VerificationFailed",
                    "ResultPublished",
                ],
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
        let id = format!("aira:{kind}:ver{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }
}

/// Tiny safe arithmetic (digits and + - * / ( )). Independent of execution-basic (CSU ↛ CSU).
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

fn claimed_matches_computed(claimed: f64, computed: f64) -> bool {
    claimed.is_finite() && computed.is_finite() && (claimed - computed).abs() <= 1e-9
}

/// Admitted capsule body from `CapsuleCompleted` second artifact ref (`#314`).
///
/// Missing or unreadable capsule is fail-closed — executor output must not substitute.
fn admitted_capsule(
    event: &EventDescriptor,
    ctx: &mut CsuExecutionContext<'_, '_>,
) -> Result<Value, String> {
    let cap_id = event
        .artifact_refs
        .get(1)
        .ok_or_else(|| "CapsuleCompleted missing admitted capsule artifact".to_string())?;
    let (_, bytes) = ctx
        .resolve_artifact(cap_id)
        .map_err(|e| format!("admitted capsule resolve: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("admitted capsule json: {e}"))
}

/// Action + expression from the admitted capsule (`#314`).
fn capsule_action_expression(capsule: &Value) -> Result<(String, String), String> {
    let action = capsule
        .get("action")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "admitted capsule missing action".to_string())?
        .to_string();
    let expression = capsule
        .get("expression")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "admitted capsule missing expression".to_string())?
        .to_string();
    Ok((action, expression))
}

/// Fail-closed if output claims a different action/expression than the capsule (`#314`).
fn output_matches_capsule(body: &Value, action: &str, expression: &str) -> Result<(), String> {
    if let Some(out_action) = body.get("action").and_then(|v| v.as_str()) {
        if out_action != action {
            return Err("output action diverges from admitted capsule".into());
        }
    }
    if let Some(out_expr) = body
        .get("expression")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        if out_expr != expression {
            return Err("output expression diverges from admitted capsule".into());
        }
    }
    Ok(())
}

/// Problem / context refs for a VRA: capsule body when CapsuleCompleted carries it, else event object_refs.
fn vra_binding_refs(
    event: &EventDescriptor,
    ctx: &CsuExecutionContext<'_, '_>,
) -> (String, String) {
    let problem_fallback = event
        .object_refs
        .first()
        .map(|r| r.as_str().to_string())
        .unwrap_or_else(|| "aira:problem:unknown".into());
    let context_fallback = String::from("aira:context:unresolved");
    let Some(cap_id) = event.artifact_refs.get(1) else {
        return (problem_fallback, context_fallback);
    };
    let Ok((_, bytes)) = ctx.resolve_artifact(cap_id) else {
        return (problem_fallback, context_fallback);
    };
    let Ok(cap) = serde_json::from_slice::<Value>(&bytes) else {
        return (problem_fallback, context_fallback);
    };
    let problem = cap
        .get("problem_statement_ref")
        .and_then(|v| v.as_str())
        .unwrap_or(problem_fallback.as_str())
        .to_string();
    let context = cap
        .get("context_ref")
        .and_then(|v| v.as_str())
        .unwrap_or(context_fallback.as_str())
        .to_string();
    (problem, context)
}

/// `artifact_hash` = SHA-256 of canonical JSON without hash/signature; `signature` over that hash string.
fn seal_vra_body(
    mut body: Value,
    tenant_csu: &AiraRef,
    publisher: &AiraRef,
) -> Result<Value, CsuHandlerError> {
    if let Some(obj) = body.as_object_mut() {
        obj.remove("artifact_hash");
        obj.remove("signature");
    }
    let bytes = canonical_json_bytes(&body).map_err(|e| CsuHandlerError {
        message: e.to_string(),
    })?;
    let hash = ContentHash::sha256_bytes(&bytes);
    let sig =
        signature_for_tenant(tenant_csu, publisher, hash.as_str().as_bytes()).map_err(|e| {
            CsuHandlerError {
                message: e.to_string(),
            }
        })?;
    let obj = body.as_object_mut().ok_or_else(|| CsuHandlerError {
        message: "VRA body must be a JSON object".into(),
    })?;
    obj.insert("artifact_hash".into(), json!(hash.as_str()));
    obj.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?,
    );
    Ok(body)
}

fn math_eval_matches_claimed(claimed: f64, expression: &str) -> bool {
    if !claimed.is_finite() {
        return false;
    }
    match math_eval_safe(expression) {
        Ok(computed) => claimed_matches_computed(claimed, computed),
        Err(_) => false,
    }
}

/// Independent of execution-basic (CSU ↛ CSU). Same `to_uppercase` as execution-basic.
fn text_matches_claimed(action: &str, claimed: &str, expression: &str) -> bool {
    match action {
        "text.echo" => claimed == expression,
        "text.uppercase" => claimed == expression.to_uppercase(),
        _ => false,
    }
}

impl Csu for VerificationBasicCsu {
    fn manifest(&self) -> &CsuManifest {
        &self.manifest
    }

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        if event.event_type != EventType::CapsuleCompleted {
            return Ok(vec![]);
        }

        let output_id = event
            .artifact_refs
            .first()
            .cloned()
            .ok_or_else(|| CsuHandlerError {
                message: "CapsuleCompleted missing output artifact".into(),
            })?;

        let (out_desc, bytes) = ctx
            .resolve_artifact(&output_id)
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        // Output Artifact must not already be a Verified Result.
        if out_desc.artifact_type == ArtifactType::VerifiedResultArtifact {
            return self.fail(ctx, event, "output already claimed as verified result");
        }

        let body: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        let capsule = match admitted_capsule(event, ctx) {
            Ok(c) => c,
            Err(msg) => return self.fail(ctx, event, &msg),
        };
        let (action, expression) = match capsule_action_expression(&capsule) {
            Ok(pair) => pair,
            Err(msg) => return self.fail(ctx, event, &msg),
        };
        if let Err(msg) = output_matches_capsule(&body, &action, &expression) {
            return self.fail(ctx, event, &msg);
        }
        // Generate-local is executed by execution-llm; do not mint a fake VERIFIED result.
        // Activate/semantic LLM verify remain later atoms.
        if action == "text.generate.local" {
            return Ok(vec![]);
        }
        let ok = match action.as_str() {
            "math.eval.safe" => match body.get("result").and_then(|v| v.as_f64()) {
                Some(claimed) => math_eval_matches_claimed(claimed, &expression),
                None => false,
            },
            "text.echo" | "text.uppercase" => match body.get("result").and_then(|v| v.as_str()) {
                Some(claimed) => text_matches_claimed(&action, claimed, &expression),
                None => false,
            },
            _ => false,
        };

        if !ok {
            return self.fail(ctx, event, "verification rejected output");
        }

        let (problem_ref, context_ref) = vra_binding_refs(event, ctx);
        let vid = self.next_id("artifact");
        let unsigned = json!({
            "result_id": vid,
            "problem_statement_ref": problem_ref,
            "context_ref": context_ref,
            "solution_refs": [output_id.as_str()],
            "evidence_refs": [],
            "verification_status": "VERIFIED",
            "confidence": 1.0,
            "scope": { "scope_type": "local", "description": "verification-basic" },
            "provenance_refs": [event.event_id.as_str(), output_id.as_str()],
            "created_at": mvp_timestamp().as_str(),
            "source_output_ref": output_id.as_str(),
            "result": body.get("result").cloned().unwrap_or(Value::Null),
            "artifact_kind": "VerifiedResultArtifact",
        });
        let verified = seal_vra_body(
            unsigned,
            &self.manifest.csu_id,
            &self.manifest.publisher_identity,
        )?;
        let payload = json_bytes(&verified);
        let vdesc = make_artifact_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &vid,
            ArtifactType::VerifiedResultArtifact,
            &payload,
            vec![event.event_id.clone(), output_id.clone()],
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.publish_artifact(vdesc.clone(), &payload)
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        let completed = make_event_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &self.next_id("event"),
            EventType::VerificationCompleted,
            event.object_refs.clone(),
            vec![vdesc.artifact_id.clone(), output_id.clone()],
            vec![event.event_id.clone()],
            None,
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.append_event(completed.clone())
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        let published = make_event_as(
            self.manifest.csu_id.clone(),
            self.manifest.publisher_identity.clone(),
            &self.next_id("event"),
            EventType::ResultPublished,
            event.object_refs.clone(),
            vec![vdesc.artifact_id.clone()],
            vec![completed.event_id.clone()],
            None,
        )
        .map_err(|e| CsuHandlerError {
            message: e.to_string(),
        })?;
        ctx.append_event(published.clone())
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;

        Ok(vec![
            CsuOutput::Artifact {
                descriptor: vdesc,
                payload,
            },
            CsuOutput::Event(completed),
            CsuOutput::Event(published),
        ])
    }
}

impl VerificationBasicCsu {
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
            EventType::VerificationFailed,
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

    fn problem() -> AiraRef {
        AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()
    }

    fn publish_capsule(store: &mut CasArtifactStore, body: Value, id: &str) -> AiraRef {
        let payload = json_bytes(&body);
        let cap = make_artifact(id, ArtifactType::ExecutionArtifact, &payload, vec![]);
        let cap_id = cap.artifact_id.clone();
        store.publish(cap, &payload).unwrap();
        cap_id
    }

    fn run_on_output(
        store: &mut CasArtifactStore,
        output_payload: Value,
        capsule_payload: Value,
    ) -> Vec<CsuOutput> {
        let cap_id = publish_capsule(store, capsule_payload, "aira:artifact:cap1");
        let mut csu = VerificationBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let payload = json_bytes(&output_payload);
        let out = make_artifact(
            "aira:artifact:out1",
            ArtifactType::ExecutionArtifact,
            &payload,
            vec![],
        );
        let oid = out.artifact_id.clone();
        store.publish(out, &payload).unwrap();
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(store),
            None,
        );
        let ev = mk(
            "aira:event:done1",
            EventType::CapsuleCompleted,
            vec![problem()],
            vec![oid, cap_id],
            vec![],
            None,
        );
        csu.on_event(&ev, &mut ctx).unwrap()
    }

    /// CapsuleCompleted with output only — no admitted capsule (`#314` fail-closed).
    fn run_on_output_without_capsule(
        store: &mut CasArtifactStore,
        output_payload: Value,
    ) -> Vec<CsuOutput> {
        let mut csu = VerificationBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let payload = json_bytes(&output_payload);
        let out = make_artifact(
            "aira:artifact:out1",
            ArtifactType::ExecutionArtifact,
            &payload,
            vec![],
        );
        let oid = out.artifact_id.clone();
        store.publish(out, &payload).unwrap();
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(store),
            None,
        );
        let ev = mk(
            "aira:event:done1",
            EventType::CapsuleCompleted,
            vec![problem()],
            vec![oid],
            vec![],
            None,
        );
        csu.on_event(&ev, &mut ctx).unwrap()
    }

    fn is_verified(outs: &[CsuOutput]) -> bool {
        outs.iter().any(|o| {
            matches!(
                o,
                CsuOutput::Artifact { descriptor, .. }
                    if descriptor.artifact_type == ArtifactType::VerifiedResultArtifact
            )
        })
    }

    fn is_failed(outs: &[CsuOutput]) -> bool {
        outs.iter().any(|o| {
            matches!(
                o,
                CsuOutput::Event(e) if e.event_type == EventType::VerificationFailed
            )
        })
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn verifies_math_output_as_verified_result() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","expression":"2+2","result":4.0}),
            json!({"action":"math.eval.safe","expression":"2+2"}),
        );
        assert!(is_verified(&outs));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::VerificationCompleted
        )));
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::ResultPublished
        )));
    }

    #[test]
    fn wrong_finite_math_result_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","expression":"2+2","result":5.0}),
            json!({"action":"math.eval.safe","expression":"2+2"}),
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }

    #[test]
    fn math_expression_from_capsule_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","result":4.0}),
            json!({"action":"math.eval.safe","expression":"2+2"}),
        );
        assert!(is_verified(&outs));
    }

    /// `#314`: substituted output expression+result must not VERIFIED even if internally consistent.
    #[test]
    fn substituted_output_expression_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","expression":"1+1","result":2.0}),
            json!({"action":"math.eval.safe","expression":"2+2"}),
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }

    #[test]
    fn missing_admitted_capsule_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output_without_capsule(
            &mut store,
            json!({"action":"math.eval.safe","expression":"2+2","result":4.0}),
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }

    #[test]
    fn finite_result_without_capsule_expression_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","result":4.0}),
            json!({"action":"math.eval.safe"}),
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }

    #[test]
    fn verifies_text_echo_output_as_verified_result() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"text.echo","expression":"hello","result":"hello"}),
            json!({"action":"text.echo","expression":"hello"}),
        );
        assert!(is_verified(&outs));
    }

    #[test]
    fn verifies_text_uppercase_output_as_verified_result() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"text.uppercase","expression":"hello","result":"HELLO"}),
            json!({"action":"text.uppercase","expression":"hello"}),
        );
        assert!(is_verified(&outs));
    }

    #[test]
    fn wrong_text_echo_result_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"text.echo","expression":"hello","result":"world"}),
            json!({"action":"text.echo","expression":"hello"}),
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }

    #[test]
    fn wrong_text_uppercase_result_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"text.uppercase","expression":"hello","result":"hello"}),
            json!({"action":"text.uppercase","expression":"hello"}),
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }

    #[test]
    fn text_echo_expression_from_capsule_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"text.echo","result":"hello"}),
            json!({"action":"text.echo","expression":"hello"}),
        );
        assert!(is_verified(&outs));
    }

    #[test]
    fn generate_local_output_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({
                "action": "text.generate.local",
                "expression": "prose",
                "result": "mock-generate:prose",
                "backend": "mock"
            }),
            json!({"action":"text.generate.local","expression":"prose"}),
        );
        assert!(!is_verified(&outs));
        assert!(!is_failed(&outs));
        assert!(outs.is_empty());
    }

    #[test]
    fn text_echo_without_capsule_expression_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"text.echo","result":"hello"}),
            json!({"action":"text.echo"}),
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }

    fn verified_payload(outs: &[CsuOutput]) -> Value {
        for o in outs {
            if let CsuOutput::Artifact {
                descriptor,
                payload,
            } = o
            {
                if descriptor.artifact_type == ArtifactType::VerifiedResultArtifact {
                    return serde_json::from_slice(payload).expect("vra json");
                }
            }
        }
        panic!("no VerifiedResultArtifact payload");
    }

    const B1_010_REQUIRED: &[&str] = &[
        "result_id",
        "problem_statement_ref",
        "context_ref",
        "solution_refs",
        "evidence_refs",
        "verification_status",
        "confidence",
        "scope",
        "provenance_refs",
        "artifact_hash",
        "signature",
        "created_at",
    ];

    #[test]
    fn verified_result_body_has_b1_010_required_keys() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","expression":"2+2","result":4.0}),
            json!({"action":"math.eval.safe","expression":"2+2"}),
        );
        assert!(is_verified(&outs));
        let vra = verified_payload(&outs);
        for key in B1_010_REQUIRED {
            assert!(vra.get(*key).is_some(), "missing required {key}");
        }
        assert_eq!(vra["result"], json!(4.0));
        assert_eq!(
            vra["problem_statement_ref"],
            json!("aira:problem:01TESTPROBLEM")
        );
        assert_eq!(vra["context_ref"], json!("aira:context:unresolved"));
        assert!(vra["artifact_hash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
        assert!(vra["signature"].get("signature_value").is_some());
        assert_eq!(vra["signature"]["algorithm"], json!("ed25519"));
    }

    #[test]
    fn verified_result_binds_refs_from_capsule() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","result":4.0}),
            json!({
                "action": "math.eval.safe",
                "expression": "2+2",
                "problem_statement_ref": "aira:problem:fromcapsule",
                "context_ref": "aira:artifact:ctxfromcapsule"
            }),
        );
        assert!(is_verified(&outs));
        let vra = verified_payload(&outs);
        assert_eq!(
            vra["problem_statement_ref"],
            json!("aira:problem:fromcapsule")
        );
        assert_eq!(vra["context_ref"], json!("aira:artifact:ctxfromcapsule"));
    }
}
