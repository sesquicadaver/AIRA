//! Local in-process Capability Routing Protocol adapter (Book II §10; QUEUE #166–#169).
//!
//! Routes by Capability → provider CSU via [`DiscoveryRegistry`]. No global routing
//! table, no Node-keyed binding. Multiple equivalent candidates when Discovery has
//! multiple providers; Policy Gate MUST ALLOW before bind (`crp.bind`). Emits
//! RouteSelected / RouteRejected / RouteFailed events when an [`EventSink`] is provided.
//! Contract: [`AIRA-RFC-0079`](../../../specs/rfc/AIRA-RFC-0079-crp-local-adapter.md).

use aira_event::{EventDescriptor, EventError, EventSink, EventType};
use aira_object::{AiraRef, ContentHash, Signature, Timestamp};
use aira_policy::{PolicyDecisionKind, PolicyGate, PolicyQuery};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::discovery::{CapabilityDescriptor, DiscoveryHit, DiscoveryRegistry};
use crate::envelope::{local_signature, ProtocolError, ScopeDescriptor};

/// Protocol version string for local CRP.
pub const CRP_VERSION: &str = "0.1";

/// Policy Gate action required before binding a route candidate (#168).
pub const CRP_BIND_ACTION: &str = "crp.bind";

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

/// Result of Policy Gate evaluation before binding a candidate (#168).
#[derive(Debug, Clone, PartialEq)]
pub enum CrpBindOutcome {
    /// Gate returned ALLOW; candidate may be used.
    Bound { candidate: Box<CrpRouteCandidate> },
    /// Gate returned DENY or REQUIRE; must not bind.
    Denied { reason: String },
}

/// In-process CRP adapter: Discovery hits → Route Candidates → Policy-gated bind.
#[derive(Debug, Default)]
pub struct LocalCrpAdapter {
    candidate_seq: u64,
    event_seq: u64,
}

impl LocalCrpAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Route a request against a local discovery registry.
    ///
    /// MUST route by Capability type / id → provider CSU. MUST NOT bind Node ids.
    /// Emits one candidate per cartesian combination of non-Node Discovery hits.
    /// On Failure, optionally appends [`EventType::RouteFailed`].
    pub fn route(
        &mut self,
        request: &CrpRouteRequest,
        discovery: &DiscoveryRegistry,
        events: Option<&mut dyn EventSink>,
    ) -> Result<CrpRouteOutcome, ProtocolError> {
        validate_route_request(request)?;

        if request.required_capabilities.is_empty() {
            let reason = "required_capabilities must be non-empty".to_string();
            emit_route_event(
                self,
                events,
                EventType::RouteFailed,
                &request.route_request_id,
                None,
                &reason,
            )?;
            return Ok(CrpRouteOutcome::Failure { reason });
        }

        let mut hop_options: Vec<Vec<CapabilityChainHop>> =
            Vec::with_capacity(request.required_capabilities.len());
        for required in &request.required_capabilities {
            if is_node_ref(&required.provider_csu) {
                return Err(ProtocolError::Schema(
                    "CRP MUST route by Capability (provider CSU), not Node".into(),
                ));
            }
            let hits: Vec<DiscoveryHit> = discovery
                .query(&required.capability_type)
                .into_iter()
                .filter(|h| {
                    !is_node_ref(&h.provider_csu)
                        && (h.capability.capability_id == required.capability_id
                            || h.capability.capability_type == required.capability_type)
                })
                .collect();
            if hits.is_empty() {
                let reason = format!(
                    "no capability route for type={} id={}",
                    required.capability_type,
                    required.capability_id.as_str()
                );
                emit_route_event(
                    self,
                    events,
                    EventType::RouteFailed,
                    &request.route_request_id,
                    None,
                    &reason,
                )?;
                return Ok(CrpRouteOutcome::Failure { reason });
            }
            let mut hops = Vec::with_capacity(hits.len());
            for hit in hits {
                if is_node_ref(&hit.provider_csu) {
                    return Err(ProtocolError::Schema(
                        "CRP MUST NOT bind a Node as provider".into(),
                    ));
                }
                let mut policy_refs = hit.capability.policy_refs.clone();
                if policy_refs.is_empty() {
                    policy_refs = request.policy_refs.clone();
                }
                hops.push(CapabilityChainHop {
                    capability_ref: hit.capability.capability_id.clone(),
                    provider_csu: hit.provider_csu.clone(),
                    role: "execute".into(),
                    policy_refs,
                });
            }
            hop_options.push(hops);
        }

        let chains = cartesian_chains(&hop_options);
        let mut candidates = Vec::with_capacity(chains.len());
        for chain in chains {
            self.candidate_seq = self.candidate_seq.saturating_add(1);
            let candidate_id =
                AiraRef::parse(format!("aira:crp:candidate:local-{}", self.candidate_seq))
                    .map_err(|e| ProtocolError::Schema(e.to_string()))?;
            let confidence = (1.0_f64).clamp(0.0, 1.0);
            candidates.push(CrpRouteCandidate {
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
            });
        }

        Ok(CrpRouteOutcome::Candidates(candidates))
    }

    /// Bind a route candidate only after Policy Gate ALLOW on [`CRP_BIND_ACTION`].
    ///
    /// DENY / REQUIRE → [`CrpBindOutcome::Denied`] (no bind) and optional
    /// [`EventType::RouteRejected`]. ALLOW → Bound + optional [`EventType::RouteSelected`].
    pub fn bind(
        &mut self,
        candidate: &CrpRouteCandidate,
        gate: &mut PolicyGate,
        events: Option<&mut dyn EventSink>,
    ) -> Result<CrpBindOutcome, ProtocolError> {
        if candidate.capability_chain.is_empty() {
            let reason = "empty capability_chain cannot bind".to_string();
            emit_route_event(
                self,
                events,
                EventType::RouteRejected,
                &candidate.route_request_id,
                Some(&candidate.route_candidate_id),
                &reason,
            )?;
            return Ok(CrpBindOutcome::Denied { reason });
        }
        for hop in &candidate.capability_chain {
            if is_node_ref(&hop.provider_csu) || is_node_ref(&hop.capability_ref) {
                return Err(ProtocolError::Schema(
                    "CRP MUST NOT bind a Node-keyed hop".into(),
                ));
            }
        }
        let provider = candidate.capability_chain[0].provider_csu.clone();
        let query = PolicyQuery {
            subject: provider.clone(),
            csu_ref: Some(provider.as_str().into()),
            action: CRP_BIND_ACTION.into(),
            object_refs: vec![candidate.route_candidate_id.clone()],
            artifact_refs: vec![],
            context_refs: vec![],
            evidence_refs: candidate.evidence_refs.clone(),
            requested_at: Timestamp::parse("2026-08-29T00:00:00Z")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
        };
        let decision = gate
            .check(query, None)
            .map_err(|e| ProtocolError::Schema(e.to_string()))?;
        match decision.decision {
            PolicyDecisionKind::Allow => {
                emit_route_event(
                    self,
                    events,
                    EventType::RouteSelected,
                    &candidate.route_request_id,
                    Some(&candidate.route_candidate_id),
                    "crp.bind ALLOW",
                )?;
                Ok(CrpBindOutcome::Bound {
                    candidate: Box::new(candidate.clone()),
                })
            }
            PolicyDecisionKind::Deny => {
                let reason = format!("{CRP_BIND_ACTION} DENY");
                emit_route_event(
                    self,
                    events,
                    EventType::RouteRejected,
                    &candidate.route_request_id,
                    Some(&candidate.route_candidate_id),
                    &reason,
                )?;
                Ok(CrpBindOutcome::Denied { reason })
            }
            PolicyDecisionKind::Require => {
                let reason = format!("{CRP_BIND_ACTION} REQUIRE (not bound)");
                emit_route_event(
                    self,
                    events,
                    EventType::RouteRejected,
                    &candidate.route_request_id,
                    Some(&candidate.route_candidate_id),
                    &reason,
                )?;
                Ok(CrpBindOutcome::Denied { reason })
            }
        }
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

fn emit_route_event(
    adapter: &mut LocalCrpAdapter,
    events: Option<&mut dyn EventSink>,
    event_type: EventType,
    request_id: &AiraRef,
    candidate_id: Option<&AiraRef>,
    payload_note: &str,
) -> Result<(), ProtocolError> {
    let Some(log) = events else {
        return Ok(());
    };
    adapter.event_seq = adapter.event_seq.saturating_add(1);
    let event_id = AiraRef::parse(format!("aira:event:crp-route-{}", adapter.event_seq))
        .map_err(|e| ProtocolError::Schema(e.to_string()))?;
    let mut object_refs = vec![request_id.clone()];
    if let Some(cid) = candidate_id {
        object_refs.push(cid.clone());
    }
    let payload_hash = ContentHash::sha256_bytes(payload_note.as_bytes());
    let ev = EventDescriptor {
        event_id,
        event_type,
        schema_version: "0.1".into(),
        producer_identity: AiraRef::parse("aira:identity:local-test")
            .map_err(|e| ProtocolError::Schema(e.to_string()))?,
        causal_refs: vec![],
        object_refs,
        artifact_refs: vec![],
        policy_refs: vec![AiraRef::parse("aira:policy:default")
            .map_err(|e| ProtocolError::Schema(e.to_string()))?],
        payload_hash,
        payload_ref: Some(payload_note.into()),
        created_at: Timestamp::parse("2026-08-29T00:00:00Z")
            .map_err(|e| ProtocolError::Schema(e.to_string()))?,
        signature: local_signature(),
    }
    .attach_canonical_signature()
    .map_err(|e| ProtocolError::Schema(e.to_string()))?;
    log.append(ev)
        .map_err(|e: EventError| ProtocolError::Schema(e.to_string()))
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

/// Cartesian product of hop options → full capability chains.
fn cartesian_chains(options: &[Vec<CapabilityChainHop>]) -> Vec<Vec<CapabilityChainHop>> {
    if options.is_empty() {
        return vec![];
    }
    let mut acc: Vec<Vec<CapabilityChainHop>> = vec![vec![]];
    for opts in options {
        let mut next = Vec::with_capacity(acc.len() * opts.len());
        for prefix in &acc {
            for hop in opts {
                let mut chain = prefix.clone();
                chain.push(hop.clone());
                next.push(chain);
            }
        }
        acc = next;
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DiscoveryRegistry;
    use crate::envelope::local_signature;

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

        let outcome = crp.route(&req, &discovery, None).unwrap();
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
        match crp.route(&req, &discovery, None).unwrap() {
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
        assert!(crp.route(&req, &discovery, None).is_err());
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
        let outcome = crp.route(&req, &discovery, None).unwrap();
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

    #[test]
    fn crp_multi_candidate_and_policy_gate_bind() {
        let mut discovery = DiscoveryRegistry::new();
        let cap_a = DiscoveryRegistry::local_capability(
            "aira:capability:math.eval.safe",
            "math.eval.safe",
            "aira:csu:execution.basic",
        )
        .unwrap();
        let cap_b = DiscoveryRegistry::local_capability(
            "aira:capability:math.eval.safe.alt",
            "math.eval.safe",
            "aira:csu:execution.alt",
        )
        .unwrap();
        discovery.register(cap_a.clone()).unwrap();
        discovery.register(cap_b).unwrap();

        let mut crp = LocalCrpAdapter::new();
        let req = LocalCrpAdapter::local_request(
            "aira:crp:request:multi",
            "aira:capsule:exec:multi",
            vec![cap_a],
            "aira:artifact:context:multi",
        )
        .unwrap();
        let outcome = crp.route(&req, &discovery, None).unwrap();
        let CrpRouteOutcome::Candidates(cands) = outcome else {
            panic!("expected candidates");
        };
        assert!(
            cands.len() >= 2,
            "expected ≥2 equivalent candidates, got {}",
            cands.len()
        );

        let mut deny_gate = PolicyGate::new(local_signature());
        match crp.bind(&cands[0], &mut deny_gate, None).unwrap() {
            CrpBindOutcome::Denied { reason } => assert!(reason.contains("DENY")),
            CrpBindOutcome::Bound { .. } => panic!("DENY gate must not bind"),
        }

        let mut allow_gate = PolicyGate::new(local_signature());
        allow_gate.allow_action(CRP_BIND_ACTION);
        match crp.bind(&cands[0], &mut allow_gate, None).unwrap() {
            CrpBindOutcome::Bound { candidate } => {
                assert_eq!(candidate.route_candidate_id, cands[0].route_candidate_id);
            }
            CrpBindOutcome::Denied { reason } => panic!("ALLOW must bind: {reason}"),
        }
    }

    #[test]
    fn crp_route_events_selected_rejected_failure() {
        use aira_event::MemoryEventLog;

        let mut discovery = DiscoveryRegistry::new();
        let cap = DiscoveryRegistry::local_capability(
            "aira:capability:math.eval.safe",
            "math.eval.safe",
            "aira:csu:execution.basic",
        )
        .unwrap();
        discovery.register(cap.clone()).unwrap();

        let mut crp = LocalCrpAdapter::new();
        let mut log = MemoryEventLog::new();

        let miss = DiscoveryRegistry::local_capability(
            "aira:capability:missing",
            "missing.type",
            "aira:csu:execution.basic",
        )
        .unwrap();
        let miss_req = LocalCrpAdapter::local_request(
            "aira:crp:request:fail-ev",
            "aira:capsule:exec:fail-ev",
            vec![miss],
            "aira:artifact:context:fail-ev",
        )
        .unwrap();
        match crp.route(&miss_req, &discovery, Some(&mut log)).unwrap() {
            CrpRouteOutcome::Failure { .. } => {}
            CrpRouteOutcome::Candidates(_) => panic!("expected failure"),
        }
        assert!(log
            .all()
            .iter()
            .any(|e| e.event_type == EventType::RouteFailed));

        let req = LocalCrpAdapter::local_request(
            "aira:crp:request:ev",
            "aira:capsule:exec:ev",
            vec![cap],
            "aira:artifact:context:ev",
        )
        .unwrap();
        let outcome = crp.route(&req, &discovery, Some(&mut log)).unwrap();
        let CrpRouteOutcome::Candidates(cands) = outcome else {
            panic!("expected candidates");
        };

        let mut deny_gate = PolicyGate::new(local_signature());
        let _ = crp.bind(&cands[0], &mut deny_gate, Some(&mut log)).unwrap();
        assert!(log
            .all()
            .iter()
            .any(|e| e.event_type == EventType::RouteRejected));

        let mut allow_gate = PolicyGate::new(local_signature());
        allow_gate.allow_action(CRP_BIND_ACTION);
        let _ = crp
            .bind(&cands[0], &mut allow_gate, Some(&mut log))
            .unwrap();
        assert!(log
            .all()
            .iter()
            .any(|e| e.event_type == EventType::RouteSelected));
    }
}
