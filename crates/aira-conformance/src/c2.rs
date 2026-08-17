//! C2 conformance suite — partial local protocols (Roadmap M13 / Analyze-46).

use std::fs;
use std::path::Path;

use aira_csu::support::make_event;
use aira_event::EventType;
use aira_protocol::{DiscoveryRegistry, EventProtocolAdapter, ProtocolStatus, EP_VERSION};
use aira_schema::SchemaRegistry;
use serde_json::Value;

use crate::report::ConformanceProfile;
use crate::runner::{fail, finalize_suite, pass, CaseResult, ConformanceError, SuiteResult};

/// Run the partial C2 local-protocol suite and emit a Conformance Report Artifact.
pub fn run_c2(artifact_root: impl AsRef<Path>) -> Result<SuiteResult, ConformanceError> {
    let cases = vec![
        test_protocol_envelope_schema(),
        test_protocol_response_schema(),
        test_identity_descriptor_schema(),
        test_discovery_returns_capability_not_node(),
        test_unsupported_version_no_side_effects(),
    ];
    finalize_suite(ConformanceProfile::C2, cases, artifact_root)
}

fn load_registry() -> Result<SchemaRegistry, ConformanceError> {
    let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR"))
        .map_err(|e| ConformanceError::Schema(e.to_string()))?;
    SchemaRegistry::load(root.join("schemas")).map_err(|e| ConformanceError::Schema(e.to_string()))
}

fn repo_root() -> Result<std::path::PathBuf, ConformanceError> {
    aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR"))
        .map_err(|e| ConformanceError::Schema(e.to_string()))
}

fn load_fixture(rel: &str) -> Result<Value, ConformanceError> {
    let path = repo_root()?.join(rel);
    let raw = fs::read_to_string(&path).map_err(|e| ConformanceError::Io(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| ConformanceError::Schema(e.to_string()))
}

fn test_protocol_envelope_schema() -> CaseResult {
    let id = "c2.protocol.envelope_schema";
    let reg = match load_registry() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let v = match load_fixture("fixtures/valid/protocol/envelope.json") {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    match reg.validate("aira:schema:protocol:envelope:0.1", &v) {
        Ok(()) => pass(id),
        Err(e) => fail(id, e.to_string()),
    }
}

fn test_protocol_response_schema() -> CaseResult {
    let id = "c2.protocol.response_schema";
    let reg = match load_registry() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let v = match load_fixture("fixtures/valid/protocol/response.json") {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    match reg.validate("aira:schema:protocol:response:0.1", &v) {
        Ok(()) => pass(id),
        Err(e) => fail(id, e.to_string()),
    }
}

fn test_identity_descriptor_schema() -> CaseResult {
    let id = "c2.identity.descriptor_schema";
    let reg = match load_registry() {
        Ok(r) => r,
        Err(e) => return fail(id, e.to_string()),
    };
    let v = match load_fixture("fixtures/valid/identity/identity-descriptor.json") {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    match reg.validate("aira:schema:identity:identity-descriptor:0.1", &v) {
        Ok(()) => pass(id),
        Err(e) => fail(id, e.to_string()),
    }
}

fn test_discovery_returns_capability_not_node() -> CaseResult {
    let id = "c2.discovery.capability_not_node";
    let mut reg = DiscoveryRegistry::new();
    let cap = match DiscoveryRegistry::local_capability(
        "aira:capability:local:c2-exec",
        "local.c2-exec",
        "aira:csu:execution-basic",
    ) {
        Ok(c) => c,
        Err(e) => return fail(id, e.to_string()),
    };
    if let Err(e) = reg.register(cap) {
        return fail(id, e.to_string());
    }
    // Must reject Node as provider.
    let bad = match DiscoveryRegistry::local_capability(
        "aira:capability:local:bad-node",
        "local.bad",
        "aira:node:somewhere",
    ) {
        Ok(c) => c,
        Err(e) => return fail(id, e.to_string()),
    };
    if reg.register(bad).is_ok() {
        return fail(id, "discovery accepted Node provider");
    }
    let hits = reg.query("local.c2-exec");
    if hits.is_empty() {
        return fail(id, "expected capability hit");
    }
    for h in &hits {
        if h.capability.provider_csu.as_str().contains(":node:") {
            return fail(id, "hit provider is Node");
        }
        if h.provider_csu.as_str().contains(":node:") {
            return fail(id, "DiscoveryHit.provider_csu is Node");
        }
    }
    pass(id)
}

fn test_unsupported_version_no_side_effects() -> CaseResult {
    let id = "c2.ep.unsupported_version_no_side_effects";
    let mut ep = EventProtocolAdapter::new();
    let event = make_event(
        "aira:event:c2-unsup",
        EventType::ProblemSubmitted,
        vec![aira_object::AiraRef::parse("aira:problem:c2").unwrap()],
        vec![],
        vec![],
        Some("hello".into()),
    );
    let before = ep.events().len();
    let (_env, resp) = match ep.publish_event(event, "9.9.9") {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if resp.status != ProtocolStatus::UnsupportedVersion {
        return fail(
            id,
            format!("expected UNSUPPORTED_VERSION, got {:?}", resp.status),
        );
    }
    if ep.events().len() != before {
        return fail(id, "unsupported version mutated event log");
    }
    // Supported version still works (sanity that adapter is alive).
    let event2 = make_event(
        "aira:event:c2-ok",
        EventType::ProblemSubmitted,
        vec![aira_object::AiraRef::parse("aira:problem:c2").unwrap()],
        vec![],
        vec![],
        Some("hello".into()),
    );
    let (_env2, resp2) = match ep.publish_event(event2, EP_VERSION) {
        Ok(v) => v,
        Err(e) => return fail(id, e.to_string()),
    };
    if resp2.status != ProtocolStatus::Accepted {
        return fail(id, format!("expected ACCEPTED, got {:?}", resp2.status));
    }
    if ep.events().len() != before + 1 {
        return fail(id, "supported publish did not append");
    }
    pass(id)
}
