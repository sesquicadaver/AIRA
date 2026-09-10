//! Immutable admission snapshot for Repair Pack 1 / QUEUE `#324`–`#325`
//! (RFC-0209 / RFC-0210).
//!
//! Captured at ProblemSubmitted admit time. Extends the request contract — not a
//! new Core ontology. HTTP/CLI/Desktop may send [`AdmissionConstraints`]; omitted
//! fields use [`AdmissionSnapshot::default_for_text`].

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
pub struct ResourceBudget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<u64>,
}

/// Fallback rules frozen at admit (default: no silent substitute).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FallbackRules {
    pub allow_model_fallback: bool,
    pub allow_placement_fallback: bool,
}

/// Request-time constraints carried by HTTP/CLI/Desktop (`#325` / RFC-0210).
///
/// Copied into an immutable [`AdmissionSnapshot`] at admit; live Settings after
/// submit must not mutate the admitted task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
}
