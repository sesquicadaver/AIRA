//! Policy query / decision types (Schema Pack §12–13).

use aira_object::{AiraRef, Signature, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Policy query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyQuery {
    pub subject: AiraRef,
    pub csu_ref: Option<String>,
    pub action: String,
    pub object_refs: Vec<AiraRef>,
    pub artifact_refs: Vec<AiraRef>,
    pub context_refs: Vec<AiraRef>,
    pub evidence_refs: Vec<AiraRef>,
    pub requested_at: Timestamp,
}

/// Decision enum — only ALLOW / DENY / REQUIRE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PolicyDecisionKind {
    Allow,
    Deny,
    Require,
}

/// Policy decision envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub decision: PolicyDecisionKind,
    pub requirements: Vec<Value>,
    pub reason_refs: Vec<AiraRef>,
    pub signature: Signature,
}
