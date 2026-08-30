//! C5 conformance scaffold — research separation + promotion gate (Phase H #180).
//!
//! Not a merge gate and not a licence to run research as operational. Exercises
//! OperationalPlane reject of research/promotion-candidate input, and promotion-
//! candidate schema fixtures (valid + unsigned/missing-source invalid).

use std::path::Path;

use aira_artifact::{ArtifactStore, ArtifactType};
use aira_csu::support::{json_bytes, make_artifact, make_event};
use aira_event::EventType;
use aira_flow::{FlowError, OperationalPlane};
use serde_json::json;

use crate::report::ConformanceProfile;
use crate::runner::{fail, finalize_suite, pass, CaseResult, ConformanceError, SuiteResult};

/// Run the local C5 scaffold (research separation + promotion gate) and emit a report.
pub fn run_c5(artifact_root: impl AsRef<Path>) -> Result<SuiteResult, ConformanceError> {
    let root = artifact_root.as_ref().join("c5-research");
    std::fs::create_dir_all(&root).map_err(|e| ConformanceError::Io(e.to_string()))?;
    let cases = vec![
        test_research_separation(&root),
        test_promotion_gate_reject(&root),
        test_promotion_candidate_schema(),
    ];
    finalize_suite(ConformanceProfile::C5, cases, artifact_root)
}

fn test_research_separation(root: &Path) -> CaseResult {
    let id = "c5.research.separation";
    let sub = root.join("separation");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }
    let mut plane = match OperationalPlane::open(&sub) {
        Ok(p) => p,
        Err(e) => return fail(id, e.to_string()),
    };
    let before = plane.events().len();

    let payload = json_bytes(&json!({"kind": "research"}));
    let research_art = make_artifact(
        "aira:artifact:c5-research-sep",
        ArtifactType::ResearchArtifact,
        &payload,
        vec![],
    );
    let art_id = research_art.artifact_id.clone();
    if let Err(e) = plane.artifacts_mut().publish(research_art, &payload) {
        return fail(id, format!("CAS publish of research must succeed: {e}"));
    }

    let research_ev = make_event(
        "aira:event:c5-research-created",
        EventType::ResearchArtifactCreated,
        vec![],
        vec![],
        vec![],
        None,
    );
    match plane.inject_and_drain(research_ev) {
        Err(FlowError::ResearchNonOperational(_)) => {}
        other => {
            return fail(
                id,
                format!("expected ResearchNonOperational for research event, got {other:?}"),
            )
        }
    }

    let published = make_event(
        "aira:event:c5-artpub-research",
        EventType::ArtifactPublished,
        vec![],
        vec![art_id],
        vec![],
        None,
    );
    match plane.inject_and_drain(published) {
        Err(FlowError::ResearchNonOperational(_)) => {}
        other => {
            return fail(
                id,
                format!("expected ResearchNonOperational for research artifact ref, got {other:?}"),
            )
        }
    }
    if plane.events().len() != before {
        return fail(id, "research input must not append to the operational log");
    }
    pass(id)
}

fn test_promotion_gate_reject(root: &Path) -> CaseResult {
    let id = "c5.promotion.gate_reject";
    let sub = root.join("promo");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }
    let mut plane = match OperationalPlane::open(&sub) {
        Ok(p) => p,
        Err(e) => return fail(id, e.to_string()),
    };
    let before = plane.events().len();
    let promo_ev = make_event(
        "aira:event:c5-promo-candidate",
        EventType::ArtifactPromotionCandidate,
        vec![],
        vec![],
        vec![],
        None,
    );
    match plane.inject_and_drain(promo_ev) {
        Err(FlowError::ResearchNonOperational(_)) => {}
        other => {
            return fail(
                id,
                format!("expected ResearchNonOperational for promotion candidate, got {other:?}"),
            )
        }
    }
    if plane.events().len() != before {
        return fail(
            id,
            "promotion-candidate event must not append to the operational log",
        );
    }
    pass(id)
}

fn test_promotion_candidate_schema() -> CaseResult {
    let id = "c5.promotion.candidate_schema";
    let root = match aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")) {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let reg = match aira_schema::SchemaRegistry::load(root.join("schemas")) {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let schema_id = "aira:schema:research:promotion-candidate:0.1";
    if let Err(e) = reg.validate_file(
        schema_id,
        root.join("fixtures/valid/research/promotion-candidate.json"),
    ) {
        return fail(id, format!("valid fixture must pass: {e}"));
    }
    if reg
        .validate_file(
            schema_id,
            root.join("fixtures/invalid/research/promotion-candidate-unsigned.json"),
        )
        .is_ok()
    {
        return fail(id, "unsigned promotion-candidate fixture must fail");
    }
    if reg
        .validate_file(
            schema_id,
            root.join("fixtures/invalid/research/promotion-candidate-missing-source.json"),
        )
        .is_ok()
    {
        return fail(id, "missing source_artifact_ref fixture must fail");
    }
    pass(id)
}
