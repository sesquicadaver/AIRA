//! Immutable admission snapshot for Repair Pack 1 / QUEUE `#324`–`#326`
//! (RFC-0209 / RFC-0210 / RFC-0211) and strict request decoding (`#333` / RFC-0217).
//!
//! Captured at ProblemSubmitted admit time. Extends the request contract — not a
//! new Core ontology. HTTP/CLI/Desktop may send [`AdmissionConstraints`]; omitted
//! fields use [`AdmissionSnapshot::default_for_text`].
//! Reuse catalog keys are derived from this snapshot (`reuse_catalog_key`, `#326`).
//! Request constraint structs use `deny_unknown_fields` so typos do not silently
//! become defaults (audit D3). Persisted [`AdmissionSnapshot`] stays permissive.

use aira_object::ContentHash;
use serde::{Deserialize, Serialize};

/// Schema id for admission snapshot OperationalArtifact payloads.
pub const ADMISSION_SNAPSHOT_SCHEMA: &str = "aira:schema:admission:snapshot:0.1";

/// Discriminator written into the artifact JSON body.
pub const ADMISSION_SNAPSHOT_KIND: &str = "admission_snapshot";

/// Placement preference frozen at admit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlacementPreference {
    /// Local execution only (default for text-only admit).
    #[default]
    Local,
    /// Remote capability may be used if policy allows.
    RemoteAllowed,
    /// Remote capability is required.
    RemoteRequired,
}

/// Whether reuse of a prior Ready Solution is permitted for this admit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReusePolicy {
    /// Reuse is allowed **after** snapshot constraints match (`#326`).
    #[default]
    AllowReuse,
    /// New execution is required (compare / measure / explicit).
    RequireNewExecution,
}

/// Optional generation knobs frozen at admit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GenerationParameters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

/// Optional resource budget frozen at admit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ResourceBudget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<u64>,
}

/// Fallback rules frozen at admit (default: no silent substitute).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FallbackRules {
    pub allow_model_fallback: bool,
    pub allow_placement_fallback: bool,
}

/// Request-time constraints carried by HTTP/CLI/Desktop (`#325` / RFC-0210).
///
/// Copied into an immutable [`AdmissionSnapshot`] at admit; live Settings after
/// submit must not mutate the admitted task.
///
/// `#333` / RFC-0217: unknown keys (e.g. `model_reff`) fail closed — they must
/// not be dropped into text-only defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AdmissionConstraints {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_model_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_content_hash: Option<String>,
    #[serde(default)]
    pub generation: GenerationParameters,
    #[serde(default)]
    pub placement: PlacementPreference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy_class: Option<String>,
    #[serde(default)]
    pub resource_budget: ResourceBudget,
    #[serde(default)]
    pub fallback: FallbackRules,
    #[serde(default)]
    pub reuse_policy: ReusePolicy,
}

/// Immutable user-constraint snapshot bound to an admitted problem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdmissionSnapshot {
    pub payload_schema: String,
    pub kind: String,
    /// Preferred model identity (content-based `model_ref` when known).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_ref: Option<String>,
    /// Allowed set for auto-selection within this admit (empty = unconstrained set).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_model_refs: Vec<String>,
    /// Content identity of the admitted problem statement text.
    pub statement_content_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_content_hash: Option<String>,
    #[serde(default)]
    pub generation: GenerationParameters,
    #[serde(default)]
    pub placement: PlacementPreference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy_class: Option<String>,
    #[serde(default)]
    pub resource_budget: ResourceBudget,
    #[serde(default)]
    pub fallback: FallbackRules,
    #[serde(default)]
    pub reuse_policy: ReusePolicy,
}

impl AdmissionSnapshot {
    /// Build a snapshot from request text + optional constraints (`#325`).
    ///
    /// Always recomputes `statement_content_hash` from `text` (caller cannot
    /// forge a mismatched hash). Schema/kind are fixed.
    pub fn from_text_and_constraints(text: &str, c: &AdmissionConstraints) -> Self {
        Self {
            payload_schema: ADMISSION_SNAPSHOT_SCHEMA.into(),
            kind: ADMISSION_SNAPSHOT_KIND.into(),
            model_ref: c.model_ref.clone(),
            allowed_model_refs: c.allowed_model_refs.clone(),
            statement_content_hash: ContentHash::sha256_bytes(text.as_bytes())
                .as_str()
                .to_string(),
            model_version: c.model_version.clone(),
            model_content_hash: c.model_content_hash.clone(),
            generation: c.generation.clone(),
            placement: c.placement,
            privacy_class: c.privacy_class.clone(),
            resource_budget: c.resource_budget.clone(),
            fallback: c.fallback.clone(),
            reuse_policy: c.reuse_policy,
        }
    }

    /// Build the default snapshot for a text-only admit (`#324`).
    ///
    /// Does not invent a selected model. Disallows silent model/placement
    /// fallback. Allows reuse pending constraint-aware keys (`#326`).
    pub fn default_for_text(text: &str) -> Self {
        Self::from_text_and_constraints(text, &AdmissionConstraints::default())
    }

    /// True when this JSON body is an admission snapshot artifact.
    pub fn is_snapshot_value(v: &serde_json::Value) -> bool {
        v.get("kind").and_then(|k| k.as_str()) == Some(ADMISSION_SNAPSHOT_KIND)
    }

    /// Verify snapshot↔text integrity at the library/HTTP submit boundary (`#335` / RFC-0219).
    ///
    /// Rejects forged `statement_content_hash`, wrong `kind` / `payload_schema` before
    /// any ProblemSubmitted effect. Builders like [`Self::from_text_and_constraints`]
    /// already set correct values; this closes the mutable-struct / hand-built path.
    pub fn verify_input_boundary(&self, problem_text: &str) -> Result<(), String> {
        if self.kind != ADMISSION_SNAPSHOT_KIND {
            return Err(format!(
                "admission boundary: kind must be {ADMISSION_SNAPSHOT_KIND}, got {}",
                self.kind
            ));
        }
        if self.payload_schema != ADMISSION_SNAPSHOT_SCHEMA {
            return Err(format!(
                "admission boundary: payload_schema must be {ADMISSION_SNAPSHOT_SCHEMA}, got {}",
                self.payload_schema
            ));
        }
        let expect = ContentHash::sha256_bytes(problem_text.as_bytes())
            .as_str()
            .to_string();
        if self.statement_content_hash != expect {
            return Err(format!(
                "admission boundary: statement_content_hash mismatch (snapshot {} != text {expect})",
                self.statement_content_hash
            ));
        }
        Ok(())
    }

    /// Validate an admission JSON factor embedded in Context (`#335`).
    ///
    /// Missing factor is allowed (legacy / text-only path without artifact). Present
    /// factor must carry correct kind + schema.
    pub fn verify_context_factor(v: &serde_json::Value) -> Result<(), String> {
        let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        if kind != ADMISSION_SNAPSHOT_KIND {
            return Err(format!(
                "admission boundary: context factor kind must be {ADMISSION_SNAPSHOT_KIND}, got {kind}"
            ));
        }
        let schema = v
            .get("payload_schema")
            .and_then(|k| k.as_str())
            .unwrap_or("");
        if schema != ADMISSION_SNAPSHOT_SCHEMA {
            return Err(format!(
                "admission boundary: context factor payload_schema must be {ADMISSION_SNAPSHOT_SCHEMA}, got {schema}"
            ));
        }
        Ok(())
    }

    /// Composite reuse-index key for this admit (`#326` / RFC-0211).
    ///
    /// Returns `None` when [`ReusePolicy::RequireNewExecution`] — callers must
    /// neither look up nor record a catalog entry. Otherwise hashes a normalized
    /// canonical serialization of snapshot identity (statement hash + model /
    /// generation / placement / privacy / budget / fallback). Text-only admits
    /// share a stable key so default C1 reuse still works; different
    /// `model_ref` (or other constraints) do not collide with text-only hits.
    pub fn reuse_catalog_key(&self) -> Option<String> {
        if self.reuse_policy == ReusePolicy::RequireNewExecution {
            return None;
        }
        let mut norm = self.clone();
        norm.allowed_model_refs.sort();
        // Policy is AllowReuse if we reached here; keep it explicit in the bytes.
        norm.reuse_policy = ReusePolicy::AllowReuse;
        let bytes = serde_json::to_vec(&norm).ok()?;
        Some(ContentHash::sha256_bytes(&bytes).as_str().to_string())
    }

    /// Fail closed when any non-default constraint is neither enforced nor
    /// explicitly unsupported (`#334` / RFC-0218 / audit D2).
    ///
    /// Matrix (non-default → outcome):
    /// - `reuse_policy` — **enforced** (reuse catalog key / skip)
    /// - `placement=local|remote_allowed` — **enforced** (local execution permitted)
    /// - `placement=remote_required` — **unsupported** (no remote cycle yet)
    /// - `model_ref` / `allowed_model_refs` on math — **unsupported**
    /// - `model_ref` / `allowed_model_refs` on generate — **enforced** at activate gate
    /// - `model_version` / `model_content_hash` — **unsupported** until verified binding
    /// - `generation.*` — **unsupported** (backends do not apply knobs yet)
    /// - `privacy_class` / `resource_budget.*` — **unsupported**
    /// - `fallback.allow_*=true` — **unsupported** (no silent substitute path)
    /// - `fallback` both false — **enforced** (default honesty)
    pub fn enforce_or_reject(&self, problem_text: &str) -> Result<(), String> {
        if self.placement == PlacementPreference::RemoteRequired {
            return Err(
                "unsupported constraint: placement=remote_required (local-only runtime; not enforced)"
                    .into(),
            );
        }
        // Local | RemoteAllowed: local execution remains valid.

        if self.generation.temperature.is_some()
            || self.generation.top_p.is_some()
            || self.generation.max_tokens.is_some()
            || self.generation.seed.is_some()
        {
            return Err(
                "unsupported constraint: generation parameters (not applied by current backends)"
                    .into(),
            );
        }
        if self.privacy_class.is_some() {
            return Err("unsupported constraint: privacy_class".into());
        }
        if self.resource_budget.max_cost.is_some() || self.resource_budget.max_latency_ms.is_some()
        {
            return Err("unsupported constraint: resource_budget".into());
        }
        if self.fallback.allow_model_fallback || self.fallback.allow_placement_fallback {
            return Err(
                "unsupported constraint: fallback allow_* (silent substitute not implemented)"
                    .into(),
            );
        }
        if self.model_version.is_some() {
            return Err("unsupported constraint: model_version".into());
        }
        if self.model_content_hash.is_some() {
            return Err("unsupported constraint: model_content_hash".into());
        }

        let math = aira_csu_reduction_basic::problem_binds_math_eval_safe(problem_text);
        if math {
            if self.model_ref.is_some() {
                return Err(
                    "unsupported constraint: model_ref on deterministic math (execute with model or omit)"
                        .into(),
                );
            }
            if !self.allowed_model_refs.is_empty() {
                return Err(
                    "unsupported constraint: allowed_model_refs on deterministic math".into(),
                );
            }
        } else {
            if !self.allowed_model_refs.is_empty() && self.model_ref.is_none() {
                return Err(
                    "unsupported constraint: allowed_model_refs without model_ref (auto-within-set not implemented)"
                        .into(),
                );
            }
            if let Some(want) = self.model_ref.as_ref() {
                if !self.allowed_model_refs.is_empty()
                    && !self.allowed_model_refs.iter().any(|a| a == want)
                {
                    return Err(format!(
                        "unsupported constraint: model_ref {want} not in allowed_model_refs"
                    ));
                }
            }
            // model_ref alone: enforced at activate gate (must match activated pointer).
        }
        // reuse_policy: always enforceable via catalog key (no reject branch).
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_for_text_is_local_no_fallback() {
        let s = AdmissionSnapshot::default_for_text("Calculate 2 + 2");
        assert_eq!(s.kind, ADMISSION_SNAPSHOT_KIND);
        assert_eq!(s.payload_schema, ADMISSION_SNAPSHOT_SCHEMA);
        assert_eq!(s.placement, PlacementPreference::Local);
        assert!(!s.fallback.allow_model_fallback);
        assert!(!s.fallback.allow_placement_fallback);
        assert_eq!(s.reuse_policy, ReusePolicy::AllowReuse);
        assert!(s.model_ref.is_none());
        assert!(s.statement_content_hash.starts_with("sha256:"));
    }

    #[test]
    fn from_constraints_overrides_model_and_hash_from_text() {
        let c = AdmissionConstraints {
            model_ref: Some("aira:model:x".into()),
            reuse_policy: ReusePolicy::RequireNewExecution,
            generation: GenerationParameters {
                temperature: Some(0.2),
                ..Default::default()
            },
            ..Default::default()
        };
        let s = AdmissionSnapshot::from_text_and_constraints("hello", &c);
        assert_eq!(s.model_ref.as_deref(), Some("aira:model:x"));
        assert_eq!(s.reuse_policy, ReusePolicy::RequireNewExecution);
        assert_eq!(s.generation.temperature, Some(0.2));
        assert_eq!(
            s.statement_content_hash,
            ContentHash::sha256_bytes(b"hello").as_str()
        );
    }

    #[test]
    fn serde_roundtrip_preserves_fields() {
        let mut s = AdmissionSnapshot::default_for_text("hello");
        s.model_ref = Some("aira:model:demo".into());
        s.allowed_model_refs = vec!["aira:model:demo".into()];
        s.reuse_policy = ReusePolicy::RequireNewExecution;
        let raw = serde_json::to_vec(&s).unwrap();
        let back: AdmissionSnapshot = serde_json::from_slice(&raw).unwrap();
        assert_eq!(back, s);
        assert!(AdmissionSnapshot::is_snapshot_value(
            &serde_json::from_slice(&raw).unwrap()
        ));
    }

    #[test]
    fn reuse_catalog_key_none_when_require_new() {
        let c = AdmissionConstraints {
            reuse_policy: ReusePolicy::RequireNewExecution,
            ..Default::default()
        };
        let s = AdmissionSnapshot::from_text_and_constraints("Calculate 2 + 2", &c);
        assert!(s.reuse_catalog_key().is_none());
    }

    #[test]
    fn reuse_catalog_key_differs_when_model_ref_differs() {
        let text = "Calculate 2 + 2";
        let a = AdmissionSnapshot::default_for_text(text);
        let b = AdmissionSnapshot::from_text_and_constraints(
            text,
            &AdmissionConstraints {
                model_ref: Some("aira:model:x".into()),
                ..Default::default()
            },
        );
        let ka = a.reuse_catalog_key().expect("allow reuse");
        let kb = b.reuse_catalog_key().expect("allow reuse");
        assert_ne!(ka, kb);
        assert_eq!(a.reuse_catalog_key(), a.reuse_catalog_key());
    }

    #[test]
    fn reuse_catalog_key_stable_under_allowed_set_order() {
        let text = "hello";
        let a = AdmissionSnapshot::from_text_and_constraints(
            text,
            &AdmissionConstraints {
                allowed_model_refs: vec!["aira:model:b".into(), "aira:model:a".into()],
                ..Default::default()
            },
        );
        let b = AdmissionSnapshot::from_text_and_constraints(
            text,
            &AdmissionConstraints {
                allowed_model_refs: vec!["aira:model:a".into(), "aira:model:b".into()],
                ..Default::default()
            },
        );
        assert_eq!(a.reuse_catalog_key(), b.reuse_catalog_key());
    }

    #[test]
    fn constraints_deny_unknown_fields() {
        let err = serde_json::from_str::<AdmissionConstraints>(r#"{"model_reff":"aira:model:x"}"#)
            .expect_err("typo must not become defaults");
        assert!(err.to_string().contains("unknown field"), "{err}");
    }

    #[test]
    fn generation_deny_unknown_fields() {
        let err = serde_json::from_str::<GenerationParameters>(r#"{"temperatur":0.1}"#)
            .expect_err("typo must fail");
        assert!(err.to_string().contains("unknown field"), "{err}");
    }

    #[test]
    fn constraints_known_fields_ok() {
        let c: AdmissionConstraints = serde_json::from_str(
            r#"{"model_ref":"aira:model:x","generation":{"temperature":0.2}}"#,
        )
        .unwrap();
        assert_eq!(c.model_ref.as_deref(), Some("aira:model:x"));
        assert_eq!(c.generation.temperature, Some(0.2));
    }

    #[test]
    fn empty_constraints_object_ok() {
        let c: AdmissionConstraints = serde_json::from_str("{}").unwrap();
        assert_eq!(c, AdmissionConstraints::default());
    }

    #[test]
    fn enforce_rejects_remote_required() {
        let mut s = AdmissionSnapshot::default_for_text("hello world");
        s.placement = PlacementPreference::RemoteRequired;
        let err = s.enforce_or_reject("hello world").unwrap_err();
        assert!(err.contains("remote_required"), "{err}");
    }

    #[test]
    fn enforce_rejects_model_ref_on_math() {
        let s = AdmissionSnapshot::from_text_and_constraints(
            "Calculate 2 + 2",
            &AdmissionConstraints {
                model_ref: Some("aira:model:x".into()),
                ..Default::default()
            },
        );
        let err = s.enforce_or_reject("Calculate 2 + 2").unwrap_err();
        assert!(err.contains("model_ref"), "{err}");
    }

    #[test]
    fn enforce_allows_text_only_math() {
        let s = AdmissionSnapshot::default_for_text("Calculate 2 + 2");
        s.enforce_or_reject("Calculate 2 + 2").unwrap();
    }

    #[test]
    fn enforce_rejects_generation_knobs() {
        let s = AdmissionSnapshot::from_text_and_constraints(
            "Summarize locally",
            &AdmissionConstraints {
                generation: GenerationParameters {
                    temperature: Some(0.2),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let err = s.enforce_or_reject("Summarize locally").unwrap_err();
        assert!(err.contains("generation"), "{err}");
    }

    #[test]
    fn enforce_allows_model_ref_on_generate_text() {
        let s = AdmissionSnapshot::from_text_and_constraints(
            "Summarize the local Problem Statement",
            &AdmissionConstraints {
                model_ref: Some("aira:model:x".into()),
                ..Default::default()
            },
        );
        s.enforce_or_reject("Summarize the local Problem Statement")
            .unwrap();
    }

    #[test]
    fn verify_input_boundary_rejects_forged_hash() {
        let mut s = AdmissionSnapshot::default_for_text("Calculate 2 + 2");
        s.statement_content_hash = "sha256:deadbeef".into();
        let err = s.verify_input_boundary("Calculate 2 + 2").unwrap_err();
        assert!(err.contains("statement_content_hash"), "{err}");
    }

    #[test]
    fn verify_input_boundary_rejects_wrong_kind() {
        let mut s = AdmissionSnapshot::default_for_text("hello");
        s.kind = "not_admission".into();
        let err = s.verify_input_boundary("hello").unwrap_err();
        assert!(err.contains("kind"), "{err}");
    }

    #[test]
    fn verify_input_boundary_ok_for_builder() {
        let s = AdmissionSnapshot::default_for_text("Calculate 2 + 2");
        s.verify_input_boundary("Calculate 2 + 2").unwrap();
    }

    #[test]
    fn verify_context_factor_rejects_bad_schema() {
        let v = serde_json::json!({
            "kind": "admission_snapshot",
            "payload_schema": "aira:schema:wrong"
        });
        let err = AdmissionSnapshot::verify_context_factor(&v).unwrap_err();
        assert!(err.contains("payload_schema"), "{err}");
    }
}
