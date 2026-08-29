//! Local in-process Capability Routing Protocol adapter (Book II §10; QUEUE #166).
//!
//! Routes by Capability → provider CSU via [`DiscoveryRegistry`]. No global routing
//! table, no Node-keyed binding. Contract: [`AIRA-RFC-0079`](../../../specs/rfc/AIRA-RFC-0079-crp-local-adapter.md).

use aira_object::{AiraRef, Signature};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::discovery::{CapabilityDescriptor, DiscoveryRegistry};
use crate::envelope::{local_signature, ProtocolError, ScopeDescriptor};

/// Protocol version string for local CRP.
pub const CRP_VERSION: &str = "0.1";

/// Constraints block on a CRP Route Request (Book II §10.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CrpRouteConstraints {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_max: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reliability_min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub federation_scope: Option<ScopeDescriptor>,
}

/// CRP Route Request (Book II §10.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrpRouteRequest {
    pub route_request_id: AiraRef,
    pub capsule_ref: AiraRef,
    pub required_capabilities: Vec<CapabilityDescriptor>,
    pub context_ref: AiraRef,
    pub policy_refs: Vec<AiraRef>,
    #[serde(default)]
    pub constraints: CrpRouteConstraints,
    pub issuer_identity: AiraRef,
    pub signature: Signature,
}

/// One hop in a capability chain (Book II §10.3) — Capability + provider CSU, never Node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityChainHop {
    pub capability_ref: AiraRef,
    pub provider_csu: AiraRef,
    pub role: String,
    pub policy_refs: Vec<AiraRef>,
}

/// CRP Route Candidate (Book II §10.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrpRouteCandidate {
    pub route_candidate_id: AiraRef,
    pub route_request_id: AiraRef,
    pub capability_chain: Vec<CapabilityChainHop>,
    pub estimated_cost: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_descriptor_ref: Option<AiraRef>,
    pub evidence_refs: Vec<AiraRef>,
    pub confidence: f64,
    pub signature: Signature,
}

/// Result of local CRP routing (B2-006: chain or failure).
#[derive(Debug, Clone, PartialEq)]
pub enum CrpRouteOutcome {
    /// One or more capability-chain candidates.
    Candidates(Vec<CrpRouteCandidate>),
    /// No admissible route (still not a Node-keyed answer).
    Failure { reason: String },
}

/// In-process CRP adapter: Discovery hits → Route Candidates.
#[derive(Debug, Default)]
pub struct LocalCrpAdapter {
    candidate_seq: u64,
}

impl LocalCrpAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Route a request against a local discovery registry.
    ///
    /// MUST route by Capability type / id → provider CSU. MUST NOT bind Node ids.
    pub fn route(
        &mut self,
        request: &CrpRouteRequest,
        discovery: &DiscoveryRegistry,
    ) -> Result<CrpRouteOutcome, ProtocolError> {
        validate_route_request(request)?;

        if request.required_capabilities.is_empty() {
            return Ok(CrpRouteOutcome::Failure {
                reason: "required_capabilities must be non-empty".into(),
            });
        }

        let mut chain: Vec<CapabilityChainHop> =
            Vec::with_capacity(request.required_capabilities.len());
        for required in &request.required_capabilities {
            if is_node_ref(&required.provider_csu) {
                return Err(ProtocolError::Schema(
                    "CRP MUST route by Capability (provider CSU), not Node".into(),
                ));
            }
            let hits = discovery.query(&required.capability_type);
            let hit = hits.into_iter().find(|h| {
                !is_node_ref(&h.provider_csu)
                    && (h.capability.capability_id == required.capability_id
                        || h.capability.capability_type == required.capability_type)
            });
            let Some(hit) = hit else {
                return Ok(CrpRouteOutcome::Failure {
                    reason: format!(
                        "no capability route for type={} id={}",
                        required.capability_type,
                        required.capability_id.as_str()
                    ),
                });
            };
            if is_node_ref(&hit.provider_csu) {
                return Err(ProtocolError::Schema(
                    "CRP MUST NOT bind a Node as provider".into(),
                ));
            }
            let mut policy_refs = hit.capability.policy_refs.clone();
            if policy_refs.is_empty() {
                policy_refs = request.policy_refs.clone();
            }
            chain.push(CapabilityChainHop {
                capability_ref: hit.capability.capability_id.clone(),
                provider_csu: hit.provider_csu.clone(),
                role: "execute".into(),
                policy_refs,
            });
        }

        self.candidate_seq = self.candidate_seq.saturating_add(1);
        let candidate_id =
            AiraRef::parse(format!("aira:crp:candidate:local-{}", self.candidate_seq))
                .map_err(|e| ProtocolError::Schema(e.to_string()))?;

        let confidence = chain
            .iter()
            .map(|_| 1.0_f64)
            .product::<f64>()
            .clamp(0.0, 1.0);

        let candidate = CrpRouteCandidate {
            route_candidate_id: candidate_id,
            route_request_id: request.route_request_id.clone(),
            capability_chain: chain,
            estimated_cost: json!({
                "amount": 0,
                "unit": "local-ops",
                "currency": null
            }),
            risk_descriptor_ref: None,
            evidence_refs: vec![],
            confidence,
            signature: local_signature(),
        };

        Ok(CrpRouteOutcome::Candidates(vec![candidate]))
    }

    /// Helper: signed local route request for tests / CLI smoke.
    pub fn local_request(
        route_request_id: &str,
        capsule_ref: &str,
        required: Vec<CapabilityDescriptor>,
        context_ref: &str,
    ) -> Result<CrpRouteRequest, ProtocolError> {
        Ok(CrpRouteRequest {
            route_request_id: AiraRef::parse(route_request_id)
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            capsule_ref: AiraRef::parse(capsule_ref)
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            required_capabilities: required,
            context_ref: AiraRef::parse(context_ref)
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            policy_refs: vec![AiraRef::parse("aira:policy:default")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?],
            constraints: CrpRouteConstraints::default(),
            issuer_identity: AiraRef::parse("aira:identity:local-test")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            signature: local_signature(),
        })
    }
}

fn is_node_ref(r: &AiraRef) -> bool {
    r.as_str().contains(":node:")
}

fn validate_route_request(request: &CrpRouteRequest) -> Result<(), ProtocolError> {
    if request.signature.signature_value.trim().is_empty() {
        return Err(ProtocolError::InvalidSignature);
    }
    for cap in &request.required_capabilities {
        if is_node_ref(&cap.capability_id) {
            return Err(ProtocolError::Schema(
                "CRP MUST NOT accept Node as capability_id".into(),
            ));
        }
        if is_node_ref(&cap.provider_csu) {
            return Err(ProtocolError::Schema(
                "CRP MUST route by Capability (provider CSU), not Node".into(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveryRegistry;

    #[test]
    fn crp_local_adapter_routes_capability_not_node() {
        let mut discovery = DiscoveryRegistry::new();
        let cap = DiscoveryRegistry::local_capability(
            "aira:capability:math.eval.safe",
            "math.eval.safe",
            "aira:csu:execution.basic",
        )
        .unwrap();
        discovery.register(cap.clone()).unwrap();

        let mut crp = LocalCrpAdapter::new();
        let req = LocalCrpAdapter::local_request(
            "aira:crp:request:local-1",
            "aira:capsule:exec:local-1",
            vec![cap],
            "aira:artifact:context:local-1",
        )
        .unwrap();

        let outcome = crp.route(&req, &discovery).unwrap();
        match outcome {
            CrpRouteOutcome::Candidates(cands) => {
                assert_eq!(cands.len(), 1);
                let c = &cands[0];
                assert_eq!(c.capability_chain.len(), 1);
                assert_eq!(
                    c.capability_chain[0].provider_csu.as_str(),
                    "aira:csu:execution.basic"
                );
                assert!(!c.capability_chain[0]
                    .provider_csu
                    .as_str()
                    .contains(":node:"));
                assert_eq!(
                    c.capability_chain[0].capability_ref.as_str(),
                    "aira:capability:math.eval.safe"
                );
            }
            CrpRouteOutcome::Failure { reason } => panic!("expected candidates, got {reason}"),
        }
    }

    #[test]
    fn crp_local_adapter_failure_when_capability_missing() {
        let discovery = DiscoveryRegistry::new();
        let cap = DiscoveryRegistry::local_capability(
            "aira:capability:missing",
            "missing.type",
            "aira:csu:execution.basic",
        )
        .unwrap();
        let mut crp = LocalCrpAdapter::new();
        let req = LocalCrpAdapter::local_request(
            "aira:crp:request:miss",
            "aira:capsule:exec:miss",
            vec![cap],
            "aira:artifact:context:miss",
        )
        .unwrap();
        match crp.route(&req, &discovery).unwrap() {
            CrpRouteOutcome::Failure { reason } => assert!(reason.contains("no capability route")),
            CrpRouteOutcome::Candidates(_) => panic!("expected failure"),
        }
    }

    #[test]
    fn crp_local_adapter_rejects_node_provider_in_request() {
        let discovery = DiscoveryRegistry::new();
        let mut cap = DiscoveryRegistry::local_capability(
            "aira:capability:bad",
            "bad.type",
            "aira:csu:execution.basic",
        )
        .unwrap();
        cap.provider_csu = AiraRef::parse("aira:node:local").unwrap();
        let mut crp = LocalCrpAdapter::new();
        let req = LocalCrpAdapter::local_request(
            "aira:crp:request:node",
            "aira:capsule:exec:node",
            vec![cap],
            "aira:artifact:context:node",
        )
        .unwrap();
        assert!(crp.route(&req, &discovery).is_err());
    }

    #[test]
    fn crp_route_candidate_matches_schema() {
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        let mut discovery = DiscoveryRegistry::new();
        let cap = DiscoveryRegistry::local_capability(
            "aira:capability:math.eval.safe",
            "math.eval.safe",
            "aira:csu:execution.basic",
        )
        .unwrap();
        discovery.register(cap.clone()).unwrap();
        let mut crp = LocalCrpAdapter::new();
        let req = LocalCrpAdapter::local_request(
            "aira:crp:request:schema",
            "aira:capsule:exec:schema",
            vec![cap],
            "aira:artifact:context:schema",
        )
        .unwrap();
        let outcome = crp.route(&req, &discovery).unwrap();
        let CrpRouteOutcome::Candidates(cands) = outcome else {
            panic!("expected candidates");
        };
        let v = serde_json::to_value(&cands[0]).unwrap();
        reg.validate("aira:schema:protocol:crp-route-candidate:0.1", &v)
            .unwrap();
        let req_v = serde_json::to_value(&req).unwrap();
        reg.validate("aira:schema:protocol:crp-route-request:0.1", &req_v)
            .unwrap();
    }
}
