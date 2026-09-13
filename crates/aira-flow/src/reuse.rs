//! Durable reuse-index catalog (`problems/reuse-index.json`, RFC-0087 / `#204`).
//!
//! Keys are admission-scoped (`#326` / RFC-0211): [`AdmissionSnapshot::reuse_catalog_key`],
//! not bare problem-text SHA. Legacy text-only keys in on-disk indexes are ignored
//! (fail-closed → re-execute).
//!
//! `#340` / RFC-0224: index hit is not enough — [`admit_reuse_candidate`] checks the
//! artifact independently (type, VERIFIED, statement/result compatibility). Foreign
//! VRA under a correct key must not become Completed.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use aira_artifact::ArtifactType;
use aira_object::ContentHash;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::admission::AdmissionSnapshot;

/// Persistent map: admission reuse key → reusable artifact id.
///
/// Field name retained for on-disk compatibility; values are no longer
/// text-only hashes after `#326`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ReuseIndex {
    pub by_content_hash: BTreeMap<String, String>,
}

pub(crate) fn load_reuse_index(path: &Path) -> Result<ReuseIndex, String> {
    if !path.exists() {
        return Ok(ReuseIndex::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

/// Look up a reusable artifact for this admit. `None` key (require-new) → miss.
pub(crate) fn lookup_artifact_id(
    path: &Path,
    admission: &AdmissionSnapshot,
) -> Result<Option<String>, String> {
    let Some(key) = admission.reuse_catalog_key() else {
        return Ok(None);
    };
    let idx = load_reuse_index(path)?;
    Ok(idx.by_content_hash.get(&key).cloned())
}

/// Record a Completed verified artifact under the admit's reuse key (`#326`).
///
/// No-op when `reuse_policy` is [`crate::ReusePolicy::RequireNewExecution`].
pub(crate) fn record_artifact_id(
    path: &Path,
    admission: &AdmissionSnapshot,
    artifact_id: &str,
) -> Result<(), String> {
    let Some(key) = admission.reuse_catalog_key() else {
        return Ok(());
    };
    let mut idx = load_reuse_index(path)?;
    idx.by_content_hash
        .entry(key)
        .or_insert_with(|| artifact_id.to_string());
    let raw = serde_json::to_string_pretty(&idx).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, raw).map_err(|e| e.to_string())
}

/// Independent reuse-candidate gate (`#340` / RFC-0224 / audit A9).
///
/// Index key match is **not** sufficient. Fail-closed → caller skips bind (re-execute).
pub(crate) fn admit_reuse_candidate(
    admission: &AdmissionSnapshot,
    statement_text: &str,
    artifact_type: ArtifactType,
    payload: &[u8],
) -> Result<(), String> {
    match artifact_type {
        ArtifactType::VerifiedResultArtifact | ArtifactType::ReadySolutionArtifact => {}
        other => {
            return Err(format!(
                "reuse candidate type {other:?} is not verified/ready (fail-closed)"
            ));
        }
    }

    let body: Value =
        serde_json::from_slice(payload).map_err(|e| format!("reuse candidate payload: {e}"))?;
    let obj = body
        .as_object()
        .ok_or_else(|| "reuse candidate must be a JSON object (fail-closed)".to_string())?;

    if let Some(status) = obj.get("verification_status").and_then(|v| v.as_str()) {
        if status != "VERIFIED" {
            return Err(format!(
                "reuse candidate verification_status={status} (want VERIFIED; fail-closed)"
            ));
        }
    }

    // Hard statement binding when stamped on the artifact (preferred).
    if let Some(h) = obj.get("statement_content_hash").and_then(|v| v.as_str()) {
        if h != admission.statement_content_hash {
            return Err(
                "reuse candidate statement_content_hash disagrees with admission (fail-closed)"
                    .into(),
            );
        }
        let expect = ContentHash::sha256_bytes(statement_text.as_bytes());
        if h != expect.as_str() {
            return Err(
                "reuse candidate statement_content_hash disagrees with statement text (fail-closed)"
                    .into(),
            );
        }
        return Ok(());
    }

    // No statement hash on payload: check result against this statement independently.
    result_compatible_with_statement(statement_text, &body)
}

/// Problem-id result lookup binding (`#341` / RFC-0225 / audit A10).
///
/// Index locator alone is insufficient. Accept when:
/// - `problem_statement_ref` equals this problem id, or
/// - `statement_content_hash` matches this problem text, or
/// - claimed `result` is independently compatible with the problem text (reuse).
pub(crate) fn artifact_binds_problem_lookup(
    problem_id: &str,
    problem_text: &str,
    body: &Value,
) -> Result<(), String> {
    if let Some(pref) = body.get("problem_statement_ref").and_then(|v| v.as_str()) {
        if pref == problem_id {
            return Ok(());
        }
    }
    if let Some(h) = body.get("statement_content_hash").and_then(|v| v.as_str()) {
        let expect = ContentHash::sha256_bytes(problem_text.as_bytes());
        if h == expect.as_str() {
            return Ok(());
        }
        return Err(
            "result artifact statement_content_hash disagrees with problem text (fail-closed)"
                .into(),
        );
    }
    result_compatible_with_statement(problem_text, body)
        .map_err(|e| format!("result artifact does not bind to problem {problem_id}: {e}"))
}

/// Whether an artifact body is an honest answer for this problem text (`#341`).
///
/// Used by reuse admit and by [`crate::local::LocalSession::get_result`] so a
/// locator swap to a foreign VRA cannot serve as this problem's result, while
/// admission-scoped reuse of a same-statement VRA (new problem id) still works.
pub(crate) fn result_compatible_with_statement(
    statement: &str,
    body: &Value,
) -> Result<(), String> {
    let result = body.get("result").ok_or_else(|| {
        "reuse candidate missing result and statement_content_hash (fail-closed)".to_string()
    })?;
    let trimmed = statement.trim();

    if looks_like_math(trimmed) {
        let expr = extract_math_expression(trimmed)?;
        let expected = math_eval_safe(&expr)?;
        let claimed = result.as_f64().ok_or_else(|| {
            "reuse candidate result is not numeric for math statement (fail-closed)".to_string()
        })?;
        if !claimed.is_finite() || (claimed - expected).abs() > 1e-9 {
            return Err(format!(
                "reuse candidate result {claimed} incompatible with statement (fail-closed)"
            ));
        }
        return Ok(());
    }

    if let Some(rest) = strip_prefix_ci(trimmed, "echo ") {
        let claimed = result.as_str().ok_or_else(|| {
            "reuse candidate result is not string for echo statement (fail-closed)".to_string()
        })?;
        if claimed != rest {
            return Err(
                "reuse candidate result incompatible with echo statement (fail-closed)".into(),
            );
        }
        return Ok(());
    }

    if let Some(rest) = strip_prefix_ci(trimmed, "uppercase ") {
        let claimed = result.as_str().ok_or_else(|| {
            "reuse candidate result is not string for uppercase statement (fail-closed)".to_string()
        })?;
        if claimed != rest.to_uppercase() {
            return Err(
                "reuse candidate result incompatible with uppercase statement (fail-closed)".into(),
            );
        }
        return Ok(());
    }

    Err(
        "reuse candidate cannot be verified against statement without statement_content_hash (fail-closed)"
            .into(),
    )
}

fn looks_like_math(statement: &str) -> bool {
    let lower = statement.to_lowercase();
    let has_digit = statement.chars().any(|c| c.is_ascii_digit());
    if lower.contains("calculate") && has_digit {
        return true;
    }
    let cleaned: String = statement.chars().filter(|c| !c.is_whitespace()).collect();
    !cleaned.is_empty()
        && cleaned.chars().any(|c| c.is_ascii_digit())
        && cleaned
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.'))
}

fn extract_math_expression(statement: &str) -> Result<String, String> {
    let trimmed = statement.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("calculate ") {
        let after = trimmed[10..].trim();
        if after.is_empty() {
            return Err("empty math expression after Calculate (fail-closed)".into());
        }
        return Ok(after.to_string());
    }
    if lower.starts_with("calculate") {
        let after = trimmed[9..].trim();
        if after.is_empty() {
            return Err("empty math expression after Calculate (fail-closed)".into());
        }
        return Ok(after.to_string());
    }
    let cleaned: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.'))
        && cleaned.chars().any(|c| c.is_ascii_digit())
    {
        return Ok(cleaned);
    }
    Err("unsupported math statement for reuse check (fail-closed)".into())
}

fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix) {
        Some(s[prefix.len()..].trim_start())
    } else {
        None
    }
}

/// Tiny safe arithmetic (digits and + - * / ( )); independent of execution CSU.
fn math_eval_safe(expr: &str) -> Result<f64, String> {
    let cleaned: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return Err("empty expression".into());
    }
    if cleaned
        .chars()
        .any(|c| !(c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '.')))
    {
        return Err("unsupported characters for reuse math check".into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::{AdmissionConstraints, ReusePolicy};
    use aira_object::ContentHash;

    /// Legacy helper: SHA of problem text alone (pre-`#326` key shape).
    fn problem_text_hash(text: &str) -> String {
        ContentHash::sha256_bytes(text.as_bytes())
            .as_str()
            .to_string()
    }

    #[test]
    fn lookup_misses_legacy_text_only_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reuse-index.json");
        let text = "Calculate 2 + 2";
        let legacy = problem_text_hash(text);
        let idx = serde_json::json!({
            "by_content_hash": { legacy: "aira:artifact:legacy" }
        });
        fs::write(&path, serde_json::to_string(&idx).unwrap()).unwrap();
        let snap = AdmissionSnapshot::default_for_text(text);
        assert!(lookup_artifact_id(&path, &snap).unwrap().is_none());
    }

    #[test]
    fn record_and_lookup_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reuse-index.json");
        let snap = AdmissionSnapshot::default_for_text("Calculate 2 + 2");
        record_artifact_id(&path, &snap, "aira:artifact:ready").unwrap();
        assert_eq!(
            lookup_artifact_id(&path, &snap).unwrap().as_deref(),
            Some("aira:artifact:ready")
        );
    }

    #[test]
    fn require_new_neither_records_nor_looks_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reuse-index.json");
        let snap = AdmissionSnapshot::from_text_and_constraints(
            "Calculate 2 + 2",
            &AdmissionConstraints {
                reuse_policy: ReusePolicy::RequireNewExecution,
                ..Default::default()
            },
        );
        record_artifact_id(&path, &snap, "aira:artifact:x").unwrap();
        assert!(!path.exists() || load_reuse_index(&path).unwrap().by_content_hash.is_empty());
        assert!(lookup_artifact_id(&path, &snap).unwrap().is_none());
    }

    #[test]
    fn admit_rejects_foreign_math_result() {
        let text = "Calculate 2 + 2";
        let snap = AdmissionSnapshot::default_for_text(text);
        let payload = serde_json::to_vec(&serde_json::json!({
            "result": 999.0,
            "verification_status": "VERIFIED",
            "confidence": 1.0,
            "scope": { "scope_type": "local", "description": "foreign" },
            "evidence_refs": [],
            "provenance_refs": []
        }))
        .unwrap();
        let err = admit_reuse_candidate(&snap, text, ArtifactType::ReadySolutionArtifact, &payload)
            .unwrap_err();
        assert!(
            err.contains("incompatible") || err.contains("fail-closed"),
            "{err}"
        );
    }

    #[test]
    fn admit_accepts_compatible_math_result() {
        let text = "Calculate 2 + 2";
        let snap = AdmissionSnapshot::default_for_text(text);
        let payload = serde_json::to_vec(&serde_json::json!({
            "result": 4.0,
            "verification_status": "VERIFIED",
            "confidence": 1.0,
            "scope": { "scope_type": "local", "description": "ok" },
            "evidence_refs": [],
            "provenance_refs": []
        }))
        .unwrap();
        admit_reuse_candidate(&snap, text, ArtifactType::ReadySolutionArtifact, &payload).unwrap();
    }

    #[test]
    fn admit_rejects_statement_hash_mismatch() {
        let text = "Calculate 2 + 2";
        let snap = AdmissionSnapshot::default_for_text(text);
        let wrong = ContentHash::sha256_bytes(b"Calculate 9 - 3");
        let payload = serde_json::to_vec(&serde_json::json!({
            "result": 4.0,
            "verification_status": "VERIFIED",
            "statement_content_hash": wrong.as_str(),
            "confidence": 1.0,
            "scope": { "scope_type": "local", "description": "hash" },
            "evidence_refs": [],
            "provenance_refs": []
        }))
        .unwrap();
        let err =
            admit_reuse_candidate(&snap, text, ArtifactType::VerifiedResultArtifact, &payload)
                .unwrap_err();
        assert!(err.contains("statement_content_hash"), "{err}");
    }
}
