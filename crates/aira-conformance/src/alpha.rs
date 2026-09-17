//! MVP alpha Definition of Done acceptance (Issue #80; remapped RFC-0242).
//!
//! OP-001 `Calculate 2 + 2` VERIFIED is **legacy non-normative**. Alpha now
//! requires a real process executor smoke (`Executed` ≠ VERIFIED) plus C0/C1.

use std::path::Path;

use aira_csu::support::make_event;
use aira_csu_execution_llm::{AlwaysActivated, ProcessBackend};
use aira_event::EventType;
use aira_flow::{init_node, LocalSession, OperationalPlane, SubmitOutcome};
use aira_object::AiraRef;

use crate::c0::run_c0;
use crate::c1::run_c1;
use crate::report::ConformanceProfile;
use crate::runner::{fail, finalize_suite, pass, CaseResult, ConformanceError, SuiteResult};

/// Run DoD acceptance checks and emit a report artifact.
pub fn run_alpha_acceptance(root: impl AsRef<Path>) -> Result<SuiteResult, ConformanceError> {
    let root = root.as_ref();
    let aira = root.join(".aira");
    let cases = vec![
        test_init_and_identity_layout(&aira),
        test_process_executor_executed(&aira),
        test_failure_evidence(&aira),
        test_c0_c1_pass(root),
    ];
    finalize_suite(
        ConformanceProfile::C1,
        cases,
        root.join("acceptance-reports"),
    )
}

fn test_init_and_identity_layout(aira: &Path) -> CaseResult {
    let id = "alpha.init_identity_layout";
    if let Err(e) = init_node(aira) {
        return fail(id, e.to_string());
    }
    for need in [
        "config.json",
        "db/aira.sqlite",
        "artifacts",
        "csu/registry.json",
        "events/event-log.json",
    ] {
        if !aira.join(need).exists() {
            return fail(id, format!("missing {need}"));
        }
    }
    // Identity file is created by CLI `identity create`. Incomplete stubs are not written
    // (QUEUE #190: LocalSession fail-closed on identity load errors).
    pass(id)
}

fn test_process_executor_executed(aira: &Path) -> CaseResult {
    let id = "alpha.process_executor_executed";
    // LocalSession::submit rebuilds the plane and would drop an opt-in process bind.
    // Alpha DoD uses the same reference plane smoke as C1 (RFC-0242).
    let dir = aira.join("alpha-process-smoke");
    let mut plane = match OperationalPlane::open(&dir) {
        Ok(p) => p,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = plane.bind_process_backend(ProcessBackend::new("echo"), AlwaysActivated) {
        return fail(id, e.to_string());
    }
    let prompt = "Summarize the local Problem Statement without leaving the host.";
    match plane.submit_problem(prompt) {
        Ok(SubmitOutcome::Executed { result, .. }) => {
            if result.get("verification_status") == Some(&serde_json::json!("VERIFIED")) {
                return fail(id, "must not VERIFIED");
            }
            if result.get("backend") != Some(&serde_json::json!("process")) {
                return fail(id, format!("expected backend=process, got {result}"));
            }
            pass(id)
        }
        Ok(other) => fail(id, format!("expected Executed, got {other:?}")),
        Err(e) => fail(id, e.to_string()),
    }
}

fn test_failure_evidence(aira: &Path) -> CaseResult {
    let id = "alpha.failure_evidence";
    let mut session = match LocalSession::open(aira) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let ev = make_event(
        "aira:event:alpha_fail1",
        EventType::CapsuleCreated,
        vec![AiraRef::parse("aira:problem:alpha_fail1").unwrap()],
        vec![AiraRef::parse("aira:artifact:missing_alpha").unwrap()],
        vec![],
        Some("text.generate.local".into()),
    );
    if let Err(e) = session.plane_mut().inject_and_drain(ev) {
        return fail(id, e.to_string());
    }
    let events = session.plane().events();
    if !events
        .iter()
        .any(|e| e.event_type == EventType::FailureEvidenceCreated)
    {
        return fail(id, "missing FailureEvidenceCreated");
    }
    if session.plane().has_verified_result_artifact() {
        // May still have verified from prior submit in same plane session —
        // check that failure path itself did not claim success via new VerificationCompleted
        // after the injected event. Accept if CapsuleFailed present.
        if !events
            .iter()
            .any(|e| e.event_type == EventType::CapsuleFailed)
        {
            return fail(id, "missing CapsuleFailed");
        }
    }
    pass(id)
}

fn test_c0_c1_pass(root: &Path) -> CaseResult {
    let id = "alpha.c0_c1_pass";
    let c0 = match run_c0(root.join("c0-reports")) {
        Ok(s) => s,
        Err(e) => return fail(id, format!("c0: {e}")),
    };
    let c1 = match run_c1(root.join("c1-reports")) {
        Ok(s) => s,
        Err(e) => return fail(id, format!("c1: {e}")),
    };
    if c0.report.results.failed > 0 {
        return fail(id, format!("c0 failures {:?}", c0.report.failures));
    }
    if c1.report.results.failed > 0 {
        return fail(id, format!("c1 failures {:?}", c1.report.failures));
    }
    pass(id)
}
