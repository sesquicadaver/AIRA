//! Shared runner types and report emission.

use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, local_identity, local_signature, make_artifact};
use aira_object::AiraRef;
use serde_json::Value;
use thiserror::Error;

use crate::report::{ConformanceProfile, ConformanceReport, FailureRecord, ResultCounters};

/// Conformance harness errors.
#[derive(Debug, Error)]
pub enum ConformanceError {
    #[error("schema: {0}")]
    Schema(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("test: {0}")]
    Test(String),
    #[error("io: {0}")]
    Io(String),
}

/// Outcome of a single conformance test case.
#[derive(Debug, Clone)]
pub enum CaseOutcome {
    Passed,
    Failed { reason: String },
    Skipped { reason: String },
}

/// Named test case result.
#[derive(Debug, Clone)]
pub struct CaseResult {
    pub test_id: String,
    pub outcome: CaseOutcome,
}

/// Suite execution result + report artifact id.
#[derive(Debug, Clone)]
pub struct SuiteResult {
    pub report: ConformanceReport,
    pub report_artifact_id: AiraRef,
    pub cases: Vec<CaseResult>,
}

/// Accumulate cases into a report and publish as immutable artifact.
pub fn finalize_suite(
    profile: ConformanceProfile,
    cases: Vec<CaseResult>,
    artifact_root: impl AsRef<Path>,
) -> Result<SuiteResult, ConformanceError> {
    let mut report = ConformanceReport::new(profile, local_signature());
    let mut counters = ResultCounters {
        total: cases.len() as u32,
        ..Default::default()
    };
    for case in &cases {
        match &case.outcome {
            CaseOutcome::Passed => counters.passed += 1,
            CaseOutcome::Failed { reason } => {
                counters.failed += 1;
                report.failures.push(FailureRecord {
                    test_id: case.test_id.clone(),
                    reason: reason.clone(),
                    evidence_refs: vec![],
                });
            }
            CaseOutcome::Skipped { .. } => counters.skipped += 1,
        }
    }
    report.results = counters;

    // Schema validate before publish.
    let value =
        serde_json::to_value(&report).map_err(|e| ConformanceError::Schema(e.to_string()))?;
    validate_report_schema(&value)?;

    let payload = json_bytes(&value);
    let art_id = format!(
        "aira:artifact:conformance_{}_{}",
        profile.as_str().to_lowercase(),
        report.results.passed
    );
    let desc = make_artifact(
        &art_id,
        ArtifactType::ConformanceArtifact,
        &payload,
        vec![local_identity()],
    );
    let art_ref = desc.artifact_id.clone();
    let mut store = CasArtifactStore::open(artifact_root.as_ref())
        .map_err(|e| ConformanceError::Artifact(e.to_string()))?;
    store
        .publish(desc, &payload)
        .map_err(|e| ConformanceError::Artifact(e.to_string()))?;

    // Immutability: second publish must fail.
    let again = make_artifact(
        &art_id,
        ArtifactType::ConformanceArtifact,
        &payload,
        vec![local_identity()],
    );
    let err = store.publish(again, &payload);
    if err.is_ok() {
        return Err(ConformanceError::Test(
            "conformance report artifact was mutable".into(),
        ));
    }

    Ok(SuiteResult {
        report,
        report_artifact_id: art_ref,
        cases,
    })
}

fn validate_report_schema(value: &Value) -> Result<(), ConformanceError> {
    let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR"))
        .map_err(|e| ConformanceError::Schema(e.to_string()))?;
    let reg = aira_schema::SchemaRegistry::load(root.join("schemas"))
        .map_err(|e| ConformanceError::Schema(e.to_string()))?;
    reg.validate("aira:schema:conformance:report:0.1", value)
        .map_err(|e| ConformanceError::Schema(e.to_string()))
}

pub(crate) fn pass(id: &str) -> CaseResult {
    CaseResult {
        test_id: id.into(),
        outcome: CaseOutcome::Passed,
    }
}

pub(crate) fn fail(id: &str, reason: impl Into<String>) -> CaseResult {
    CaseResult {
        test_id: id.into(),
        outcome: CaseOutcome::Failed {
            reason: reason.into(),
        },
    }
}
