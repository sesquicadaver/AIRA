//! C3 conformance scaffold — local federation + capability advertisement
//! + policy-scoped IO deny + CRP node-keyed reject (Phase G #141; Phase H #161/#163/#167).
//!
//! Not production federated wire protocol; exercises descriptor verify, join,
//! leave, re-join, local CAP persist, federation export deny, and CRP capability≠node
//! from `aira-protocol`.

use std::path::Path;

use aira_object::{AiraRef, Signature, TrustAuditAction, TrustAuditLog};
use aira_protocol::{
    check_federation_transfer, descriptor_canonical_bytes, join_federation, leave_federation,
    load_federation_membership, membership_path, verify_federation_descriptor,
    CapabilityAdvertisementStore, DiscoveryRegistry, FederationDescriptor, FederationTransferKind,
    LocalCrpAdapter, TransferCheckOutcome, FEDERATION_DESCRIPTOR_DOMAIN,
};
use ed25519_dalek::SigningKey;

use crate::report::ConformanceProfile;
use crate::runner::{fail, finalize_suite, pass, CaseResult, ConformanceError, SuiteResult};

/// Run the local C3 scaffold (≥6 named cases) and emit a Conformance Report Artifact.
pub fn run_c3(artifact_root: impl AsRef<Path>) -> Result<SuiteResult, ConformanceError> {
    let root = artifact_root.as_ref().join("c3-federation");
    std::fs::create_dir_all(&root).map_err(|e| ConformanceError::Io(e.to_string()))?;
    let cases = vec![
        test_federation_descriptor_verify(),
        test_federation_join_membership(&root),
        test_federation_leave_clears(&root),
        test_federation_rejoin_after_leave(&root),
        test_capability_advertisement(&root),
        test_federation_export_deny(&root),
        test_crp_reject_node_route(),
    ];
    finalize_suite(ConformanceProfile::C3, cases, artifact_root)
}

fn signed_descriptor(id: &str, fed: &str, seed: u8) -> FederationDescriptor {
    let sk = SigningKey::from_bytes(&[seed; 32]);
    let pk = hex::encode(sk.verifying_key().to_bytes());
    let mut d = FederationDescriptor {
        schema: FEDERATION_DESCRIPTOR_DOMAIN.into(),
        federation_id: fed.into(),
        federation_type: "private".into(),
        identity_ref: id.into(),
        public_key_hex: pk,
        signature: Signature {
            algorithm: "ed25519".into(),
            key_ref: AiraRef::parse(id).expect("identity ref"),
            signature_value: String::new(),
        },
    };
    d.signature = aira_object::sign_with_key(
        AiraRef::parse(id).expect("identity ref"),
        &sk,
        &descriptor_canonical_bytes(&d),
    );
    d
}

fn test_federation_descriptor_verify() -> CaseResult {
    let id = "c3.federation.descriptor_verify";
    let desc = signed_descriptor("aira:identity:c3-fed", "aira:federation:c3-verify", 31);
    if let Err(e) = verify_federation_descriptor(&desc) {
        return fail(id, e.to_string());
    }
    let mut bad = desc.clone();
    bad.signature.signature_value.clear();
    if verify_federation_descriptor(&bad).is_ok() {
        return fail(id, "unsigned descriptor must be rejected");
    }
    pass(id)
}

fn test_federation_join_membership(root: &Path) -> CaseResult {
    let id = "c3.federation.join_membership";
    let sub = root.join("join");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }
    let desc = signed_descriptor("aira:identity:c3-join", "aira:federation:c3-join", 32);
    let out = match join_federation(&sub, &desc) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if out.already_member {
        return fail(id, "first join must not be idempotent");
    }
    if !membership_path(&sub).is_file() {
        return fail(id, "membership.json missing after join");
    }
    let loaded = match load_federation_membership(&sub) {
        Ok(Some(m)) => m,
        Ok(None) => return fail(id, "membership not loaded"),
        Err(e) => return fail(id, e.to_string()),
    };
    if loaded.federation_id != desc.federation_id {
        return fail(id, "membership federation_id mismatch");
    }
    pass(id)
}

fn test_federation_leave_clears(root: &Path) -> CaseResult {
    let id = "c3.federation.leave_clears";
    let sub = root.join("leave");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }
    let desc = signed_descriptor("aira:identity:c3-leave", "aira:federation:c3-leave", 33);
    if let Err(e) = join_federation(&sub, &desc) {
        return fail(id, e.to_string());
    }
    let out = match leave_federation(&sub) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if !out.was_member {
        return fail(id, "leave expected was_member");
    }
    if membership_path(&sub).exists() {
        return fail(id, "membership.json must be removed after leave");
    }
    pass(id)
}

fn test_federation_rejoin_after_leave(root: &Path) -> CaseResult {
    let id = "c3.federation.rejoin_after_leave";
    let sub = root.join("rejoin");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }
    let first = signed_descriptor("aira:identity:c3-rejoin-a", "aira:federation:c3-a", 34);
    let second = signed_descriptor("aira:identity:c3-rejoin-b", "aira:federation:c3-b", 35);
    if let Err(e) = join_federation(&sub, &first) {
        return fail(id, e.to_string());
    }
    if let Err(e) = leave_federation(&sub) {
        return fail(id, e.to_string());
    }
    let out = match join_federation(&sub, &second) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if out.already_member {
        return fail(id, "post-leave join must not be idempotent");
    }
    if out.membership.federation_id != second.federation_id {
        return fail(id, "rejoin federation_id mismatch");
    }
    let audit = match TrustAuditLog::load(&sub) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if audit.len() < 3 {
        return fail(id, "expected join/leave/rejoin audit entries");
    }
    if !audit
        .iter()
        .any(|e| e.action == TrustAuditAction::FederationJoin)
    {
        return fail(id, "missing federation_join audit");
    }
    if !audit
        .iter()
        .any(|e| e.action == TrustAuditAction::FederationLeave)
    {
        return fail(id, "missing federation_leave audit");
    }
    pass(id)
}

/// B2-005 local CAP: persist advertisement, reopen, reject Node-keyed provider (#161).
fn test_capability_advertisement(root: &Path) -> CaseResult {
    let id = "c3.capability.advertisement";
    let sub = root.join("capability-ad");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }
    let mut store = CapabilityAdvertisementStore::new();
    let ad = match CapabilityAdvertisementStore::local_advertisement(
        "aira:capability-ad:c3:execution-basic",
        "local.execution-basic",
        "aira:csu:execution-basic",
    ) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    // B2-005 MUST fields
    if ad.provider_csu.as_str().is_empty()
        || ad.capability.capability_type.is_empty()
        || ad.capability.scope.scope_type.is_empty()
        || ad.policy_refs.is_empty()
        || ad.signature.signature_value.trim().is_empty()
    {
        return fail(id, "B2-005 required fields missing on local advertisement");
    }
    if let Err(e) = store.register(ad) {
        return fail(id, e.to_string());
    }
    if let Err(e) = store.save(&sub) {
        return fail(id, e.to_string());
    }
    if !CapabilityAdvertisementStore::path(&sub).is_file() {
        return fail(id, "advertisements.json missing after save");
    }
    let loaded = match CapabilityAdvertisementStore::load(&sub) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if !loaded.contains("aira:capability-ad:c3:execution-basic") {
        return fail(id, "advertisement not loaded after reopen");
    }
    let mut bad = match CapabilityAdvertisementStore::local_advertisement(
        "aira:capability-ad:c3:node-keyed",
        "local.bad",
        "aira:csu:execution-basic",
    ) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    bad.provider_csu = match AiraRef::parse("aira:node:local") {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    let mut reject = CapabilityAdvertisementStore::new();
    if reject.register(bad).is_ok() {
        return fail(id, "Node-keyed provider_csu must be rejected");
    }
    pass(id)
}

/// PRIV-003 / Book II §18: federation export deny-by-default + trust audit (#162/#163).
fn test_federation_export_deny(root: &Path) -> CaseResult {
    let id = "c3.federation.export_deny";
    let sub = root.join("export-deny");
    if let Err(e) = std::fs::create_dir_all(&sub) {
        return fail(id, e.to_string());
    }
    let subject = "aira:artifact:c3:cross-fed:1";
    let outcome =
        match check_federation_transfer(sub.as_path(), FederationTransferKind::Export, subject) {
            Ok(v) => v,
            Err(e) => return fail(id, e.to_string()),
        };
    match outcome {
        TransferCheckOutcome::Deny { reason } => {
            if !reason.contains("export") {
                return fail(id, format!("deny reason should mention export: {reason}"));
            }
        }
        TransferCheckOutcome::Allow => return fail(id, "export must deny by default"),
    }
    let audit = match TrustAuditLog::load(&sub) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    let deny_entries: Vec<_> = audit
        .iter()
        .filter(|e| e.action == TrustAuditAction::FederationExportDeny)
        .collect();
    if deny_entries.is_empty() {
        return fail(id, "missing federation_export_deny audit");
    }
    if deny_entries.iter().any(|e| e.subject_id != subject) {
        return fail(id, "audit subject_id mismatch");
    }
    pass(id)
}

/// Book II §10.4 / B2-006 precursor: CRP MUST NOT route by Node (#167).
fn test_crp_reject_node_route() -> CaseResult {
    let id = "c3.crp.reject_node_route";
    let discovery = DiscoveryRegistry::new();
    let mut cap = match DiscoveryRegistry::local_capability(
        "aira:capability:c3:node-keyed",
        "c3.node.keyed",
        "aira:csu:execution.basic",
    ) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    cap.provider_csu = match AiraRef::parse("aira:node:local") {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    let mut crp = LocalCrpAdapter::new();
    let req = match LocalCrpAdapter::local_request(
        "aira:crp:request:c3-node",
        "aira:capsule:exec:c3-node",
        vec![cap],
        "aira:artifact:context:c3-node",
    ) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    match crp.route(&req, &discovery, None) {
        Ok(_) => fail(id, "node-keyed CRP route must fail"),
        Err(e) => {
            let msg = e.to_string();
            if !(msg.contains("Node") || msg.contains("node")) {
                return fail(id, format!("error should mention Node: {msg}"));
            }
            pass(id)
        }
    }
}
