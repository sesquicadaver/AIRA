//! Conformance Report Artifact types (Issue #63).

use aira_object::{AiraRef, Signature};
use serde::{Deserialize, Serialize};

/// Profile under test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConformanceProfile {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
}

impl ConformanceProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::C0 => "C0",
            Self::C1 => "C1",
            Self::C2 => "C2",
            Self::C3 => "C3",
            Self::C4 => "C4",
            Self::C5 => "C5",
        }
    }
}

/// Implementation under test.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImplementationInfo {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

/// AIRA standard identity for the report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiraInfo {
    pub standard_version: String,
    pub profile: ConformanceProfile,
}

/// Aggregate result counters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResultCounters {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub unsupported: u32,
    pub invalid: u32,
}

/// Single failure record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailureRecord {
    pub test_id: String,
    pub reason: String,
    pub evidence_refs: Vec<AiraRef>,
}

/// Schema-aligned Conformance Report body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConformanceReport {
    pub implementation: ImplementationInfo,
    pub aira: AiraInfo,
    pub results: ResultCounters,
    pub failures: Vec<FailureRecord>,
    pub signature: Signature,
}

impl ConformanceReport {
    /// Build a report shell for a profile.
    pub fn new(profile: ConformanceProfile, signature: Signature) -> Self {
        Self {
            implementation: ImplementationInfo {
                name: "aira-node".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                commit: None,
            },
            aira: AiraInfo {
                standard_version: "0.1".into(),
                profile,
            },
            results: ResultCounters::default(),
            failures: vec![],
            signature,
        }
    }
}
