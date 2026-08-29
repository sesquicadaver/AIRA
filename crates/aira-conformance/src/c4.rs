//! C4 conformance scaffold — local settlement audit receipts (Phase H #175).
//!
//! Not a blockchain ledger. Exercises receipt emit/verify-on-read, B2-011 privacy
//! reject, and linking a receipt to a prior CRP route candidate id.

use std::path::Path;

use aira_object::AiraRef;
use aira_protocol::{
    validate_settlement_privacy, CrpRouteOutcome, DiscoveryRegistry, LocalCrpAdapter,
    SettlementReceiptStore,
};
use serde_json::Value;

use crate::report::ConformanceProfile;
use crate::runner::{fail, finalize_suite, pass, CaseResult, ConformanceError, SuiteResult};

/// Run the local C4 scaffold (minimal settlement cases) and emit a Conformance Report.
pub fn run_c4(artifact_root: impl AsRef<Path>) -> Result<SuiteResult, ConformanceError> {
    let root = artifact_root.as_ref().join("c4-settlement");
    std::fs::create_dir_all(&root).map_err(|e| ConformanceError::Io(e.to_string()))?;
    let cases = vec![
        test_settlement_receipt_emit_verify(&root),
        test_settlement_privacy_reject(),
        test_settlement_link_prior_route(&root),
    ];
    finalize_suite(ConformanceProfile::C4, cases, artifact_root)
}

fn test_settlement_receipt_emit_verify(root: &Path) -> CaseResult {
    let id = "c4.settlement.receipt_emit_verify";
    let sub = root.join("emit");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }
    let mut store = match SettlementReceiptStore::open_or_create(&sub) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let receipt = match SettlementReceiptStore::local_receipt("aira:settlement:receipt:c4-emit") {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = store.append(receipt.clone()) {
        return fail(id, e.to_string());
    }
    drop(store);
    let reopened = match SettlementReceiptStore::open(&sub) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    let got = match reopened.get(&receipt.receipt_id) {
        Ok(Some(r)) => r,
        Ok(None) => return fail(id, "receipt missing after reopen"),
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = got.verify_canonical() {
        return fail(id, e.to_string());
    }
    if got.privacy_class.trim().is_empty() {
        return fail(id, "privacy_class must be present");
    }
    pass(id)
}

fn test_settlement_privacy_reject() -> CaseResult {
    let id = "c4.settlement.privacy_reject";
    let mut v = match SettlementReceiptStore::local_receipt("aira:settlement:receipt:c4-priv") {
        Ok(r) => match serde_json::to_value(r) {
            Ok(v) => v,
            Err(e) => return fail(id, e.to_string()),
        },
        Err(e) => return fail(id, e.to_string()),
    };
    v.as_object_mut()
        .unwrap()
        .insert("raw_prompt".into(), Value::String("leak".into()));
    match validate_settlement_privacy(&v) {
        Ok(()) => fail(id, "raw_prompt must be rejected (B2-011 / PRIV-001)"),
        Err(e) => {
            if !e.to_string().contains("raw_prompt") {
                return fail(id, format!("error should name raw_prompt: {e}"));
            }
            pass(id)
        }
    }
}

fn test_settlement_link_prior_route(root: &Path) -> CaseResult {
    let id = "c4.settlement.link_prior_route";
    let sub = root.join("link");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }

    let mut discovery = DiscoveryRegistry::new();
    let cap = match DiscoveryRegistry::local_capability(
        "aira:capability:c4:math.eval.safe",
        "math.eval.safe",
        "aira:csu:execution.basic",
    ) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = discovery.register(cap.clone()) {
        return fail(id, e.to_string());
    }
    let mut crp = LocalCrpAdapter::new();
    let req = match LocalCrpAdapter::local_request(
        "aira:crp:request:c4-link",
        "aira:capsule:exec:c4-link",
        vec![cap],
        "aira:artifact:context:c4-link",
    ) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    let candidate_id = match crp.route(&req, &discovery, None) {
        Ok(CrpRouteOutcome::Candidates(cands)) if !cands.is_empty() => {
            cands[0].route_candidate_id.clone()
        }
        Ok(other) => return fail(id, format!("expected CRP candidates, got {other:?}")),
        Err(e) => return fail(id, e.to_string()),
    };

    let mut receipt = match SettlementReceiptStore::local_receipt("aira:settlement:receipt:c4-link")
    {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    receipt.execution_or_artifact_ref = candidate_id.clone();
    receipt = match receipt.attach_canonical_signature() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };

    let mut store = match SettlementReceiptStore::open_or_create(&sub) {
        Ok(s) => s,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = store.append(receipt) {
        return fail(id, e.to_string());
    }
    let got = match store.get(&AiraRef::parse("aira:settlement:receipt:c4-link").unwrap()) {
        Ok(Some(r)) => r,
        Ok(None) => return fail(id, "linked receipt missing"),
        Err(e) => return fail(id, e.to_string()),
    };
    if got.execution_or_artifact_ref != candidate_id {
        return fail(
            id,
            "execution_or_artifact_ref must link prior CRP candidate",
        );
    }
    pass(id)
}
