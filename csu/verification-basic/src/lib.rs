//! Verification-basic CSU (Issue #44).
//!
//! Distinguishes Output Artifact from Verified Result Artifact.
//! `#187`: `math.eval.safe` is independently evaluated; a wrong finite result is not VERIFIED.

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, json_bytes, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::AiraRef;
use serde_json::{json, Value};

/// Deterministic verification CSU.
pub struct VerificationBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: u64,
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
            run_nonce: 0,
        }
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

fn math_expression(
    body: &Value,
    event: &EventDescriptor,
    ctx: &mut CsuExecutionContext<'_, '_>,
) -> Option<String> {
    if let Some(expr) = body
        .get("expression")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        return Some(expr.to_string());
    }
    let cap_id = event.artifact_refs.get(1)?;
    let (_, bytes) = ctx.resolve_artifact(cap_id).ok()?;
    let cap: Value = serde_json::from_slice(&bytes).ok()?;
    cap.get("expression")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn math_eval_matches_claimed(
    body: &Value,
    event: &EventDescriptor,
    ctx: &mut CsuExecutionContext<'_, '_>,
) -> bool {
    let Some(claimed) = body.get("result").and_then(|v| v.as_f64()) else {
        return false;
    };
    if !claimed.is_finite() {
        return false;
    }
    let Some(expr) = math_expression(body, event, ctx) else {
        return false;
    };
    match math_eval_safe(&expr) {
        Ok(computed) => claimed_matches_computed(claimed, computed),
        Err(_) => false,
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
        let action = body.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let ok = match action {
            "math.eval.safe" => math_eval_matches_claimed(&body, event, ctx),
            "text.echo" | "text.uppercase" => body.get("result").and_then(|v| v.as_str()).is_some(),
            _ => false,
        };

        if !ok {
            return self.fail(ctx, event, "verification rejected output");
        }

        let verified = json!({
            "result": body.get("result").cloned().unwrap_or(Value::Null),
            "verification_status": "VERIFIED",
            "confidence": 1.0,
            "scope": { "scope_type": "local", "description": "verification-basic" },
            "source_output_ref": output_id.as_str(),
            "artifact_kind": "VerifiedResultArtifact",
            "evidence_refs": [],
            "provenance_refs": [event.event_id.as_str(), output_id.as_str()]
        });
        let payload = json_bytes(&verified);
        let vid = self.next_id("artifact");
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

    fn run_on_output(
        store: &mut CasArtifactStore,
        output_payload: Value,
        extra_refs: Vec<AiraRef>,
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
        let mut refs = vec![oid];
        refs.extend(extra_refs);
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
            refs,
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
            vec![],
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
            vec![],
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }

    #[test]
    fn math_expression_from_capsule_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let cap_body = json!({"action":"math.eval.safe","expression":"2+2"});
        let cap_payload = json_bytes(&cap_body);
        let cap = make_artifact(
            "aira:artifact:cap1",
            ArtifactType::ExecutionArtifact,
            &cap_payload,
            vec![],
        );
        let cap_id = cap.artifact_id.clone();
        store.publish(cap, &cap_payload).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","result":4.0}),
            vec![cap_id],
        );
        assert!(is_verified(&outs));
    }

    #[test]
    fn finite_result_without_expression_is_not_verified() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let outs = run_on_output(
            &mut store,
            json!({"action":"math.eval.safe","result":4.0}),
            vec![],
        );
        assert!(!is_verified(&outs));
        assert!(is_failed(&outs));
    }
}
