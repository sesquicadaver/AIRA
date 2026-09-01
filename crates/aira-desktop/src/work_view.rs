//! Human-first view of `POST /v1/problems` JSON (Work tab).
//!
//! The node returns a Verified Result Artifact envelope. The product answer is
//! `result.result` plus `status` / `verification_status` — not hashes first.

use serde_json::Value;

/// Parsed Work-tab result: lead with the human answer; keep VRA JSON secondary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkResultView {
    /// `result.result` (or a short summary of an object/array).
    pub answer: String,
    /// Envelope `status` (`completed`, `needs_human_collapse`, …).
    pub status: String,
    /// VRA `verification_status` when present (`VERIFIED`, …).
    pub verification_status: Option<String>,
    pub problem_id: Option<String>,
    pub verified_artifact_id: Option<String>,
    pub field_artifact_id: Option<String>,
    /// Full original JSON (collapsed Details in the GUI).
    pub details_json: String,
}

#[cfg(test)]
impl WorkResultView {
    /// Primary chrome: answer, status, verification — no hashes or signatures.
    fn human_lead(&self) -> String {
        let mut lines = Vec::with_capacity(3);
        if !self.answer.is_empty() {
            lines.push(self.answer.clone());
        }
        if !self.status.is_empty() {
            lines.push(self.status.clone());
        }
        if let Some(vs) = &self.verification_status {
            lines.push(vs.clone());
        }
        lines.join("\n")
    }
}

/// Format a `/v1/problems` JSON value for the Desktop Work tab.
pub fn format_work_result(v: &Value) -> WorkResultView {
    let status = opt_str(v, "status").unwrap_or_else(|| "unknown".into());
    let problem_id = opt_str(v, "problem_id");
    let verified_artifact_id = opt_str(v, "verified_artifact_id");
    let field_artifact_id = opt_str(v, "field_artifact_id");
    let verification_status = verification_status_of(v);
    let answer = extract_answer(v).map(summarize_value).unwrap_or_default();
    let details_json = serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string());
    WorkResultView {
        answer,
        status,
        verification_status,
        problem_id,
        verified_artifact_id,
        field_artifact_id,
        details_json,
    }
}

fn opt_str(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn verification_status_of(v: &Value) -> Option<String> {
    v.get("result")
        .and_then(|r| opt_str(r, "verification_status"))
        .or_else(|| opt_str(v, "verification_status"))
}

/// Human payload: nested VRA `result.result`, or a primitive top-level `result`.
fn extract_answer(v: &Value) -> Option<&Value> {
    let inner = v.get("result")?;
    if inner.is_object() {
        return inner.get("result");
    }
    if inner.is_null() {
        return None;
    }
    Some(inner)
}

fn summarize_value(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(a) => {
            if a.is_empty() {
                "[]".into()
            } else if a.len() <= 4 && a.iter().all(is_scalar) {
                let parts: Vec<String> = a.iter().map(summarize_value).collect();
                format!("[{}]", parts.join(", "))
            } else {
                format!("[{} items]", a.len())
            }
        }
        Value::Object(m) => {
            if m.is_empty() {
                "{}".into()
            } else if m.len() <= 3 && m.values().all(is_scalar) {
                serde_json::to_string(v).unwrap_or_else(|_| format!("{{{} keys}}", m.len()))
            } else {
                format!("{{{} keys}}", m.len())
            }
        }
    }
}

fn is_scalar(v: &Value) -> bool {
    matches!(
        v,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Envelope shaped like the Work-tab dump: human answer buried as `result.result`.
    fn completed_vra_like_user_paste() -> Value {
        json!({
            "status": "completed",
            "problem_id": "aira:problem:01TESTPROBLEMDEADBEEFCAFE",
            "verified_artifact_id": "aira:artifact:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "result": {
                "result_id": "aira:result:01R1",
                "problem_statement_ref": "aira:problem:01TESTPROBLEMDEADBEEFCAFE",
                "context_ref": "aira:context:01CTX",
                "solution_refs": [
                    "aira:artifact:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                ],
                "evidence_refs": ["aira:evidence:01EV1"],
                "verification_status": "VERIFIED",
                "confidence": 1.0,
                "scope": { "scope_type": "local", "description": "verification-basic" },
                "provenance_refs": ["aira:event:01E1"],
                "artifact_hash": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
                "signature": {
                    "algorithm": "ed25519",
                    "key_ref": "aira:identity:local-test",
                    "signature_value": "TESTSIGHASHNOTFORHUMANS"
                },
                "created_at": "2026-07-10T12:00:00Z",
                "source_output_ref": "aira:artifact:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "result": 4.0,
                "artifact_kind": "VerifiedResultArtifact"
            }
        })
    }

    #[test]
    fn completed_vra_leads_with_answer_and_verified_not_hashes() {
        let v = completed_vra_like_user_paste();
        let view = format_work_result(&v);
        let lead = view.human_lead();

        assert!(
            view.answer.contains('4'),
            "answer must surface 4 / 4.0, got {:?}",
            view.answer
        );
        assert!(
            lead.contains("4.0") || lead.contains('4'),
            "human lead must contain 4 / 4.0 before hashes: {lead:?}"
        );
        assert_eq!(view.status, "completed");
        assert_eq!(view.verification_status.as_deref(), Some("VERIFIED"));
        assert!(
            lead.contains("VERIFIED"),
            "lead must show VERIFIED: {lead:?}"
        );

        let hash_needles = [
            "sha256:",
            "TESTSIGHASHNOTFORHUMANS",
            "DEADBEEF",
            "signature_value",
            "artifact_hash",
        ];
        for needle in hash_needles {
            assert!(
                !lead.contains(needle),
                "human lead must not require reading {needle}: {lead:?}"
            );
        }

        assert!(view.details_json.contains("sha256:"));
        assert!(view.details_json.contains("TESTSIGHASHNOTFORHUMANS"));
        assert_eq!(
            view.problem_id.as_deref(),
            Some("aira:problem:01TESTPROBLEMDEADBEEFCAFE")
        );
        assert!(view
            .verified_artifact_id
            .as_deref()
            .unwrap_or("")
            .starts_with("aira:artifact:"));
    }

    #[test]
    fn primitive_result_still_surfaces_answer() {
        let v = json!({ "status": "completed", "result": 4.0 });
        let view = format_work_result(&v);
        assert!(view.answer.contains('4'));
        assert_eq!(view.status, "completed");
        assert!(view.verification_status.is_none());
    }

    #[test]
    fn needs_human_collapse_has_no_fake_answer() {
        let v = json!({
            "status": "needs_human_collapse",
            "problem_id": "aira:problem:x",
            "field_artifact_id": "aira:artifact:field"
        });
        let view = format_work_result(&v);
        assert!(view.answer.is_empty());
        assert_eq!(view.status, "needs_human_collapse");
        assert_eq!(
            view.field_artifact_id.as_deref(),
            Some("aira:artifact:field")
        );
        assert!(!view.human_lead().contains("sha256"));
    }

    #[test]
    fn string_and_object_answers_are_summarized() {
        let s = format_work_result(&json!({
            "status": "completed",
            "result": { "result": "hello", "verification_status": "VERIFIED" }
        }));
        assert_eq!(s.answer, "hello");

        let obj = format_work_result(&json!({
            "status": "completed",
            "result": {
                "result": { "a": 1, "b": 2, "c": 3, "d": 4 },
                "verification_status": "VERIFIED"
            }
        }));
        assert_eq!(obj.answer, "{4 keys}");
    }
}
