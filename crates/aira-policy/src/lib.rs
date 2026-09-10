//! AIRA Policy Gate (Issue Set Epic 4 / #33).
//!
//! Returns only ALLOW | DENY | REQUIRE. Unknown controlled actions default to DENY.

mod gate;
mod types;

pub use gate::{PolicyError, PolicyGate};
pub use types::{PolicyDecision, PolicyDecisionKind, PolicyQuery};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_event::{EventType, MemoryEventLog};
    use aira_object::{AiraRef, Signature, Timestamp};

    fn query(action: &str) -> PolicyQuery {
        PolicyQuery {
            subject: AiraRef::parse("aira:csu:ctx.basic").unwrap(),
            csu_ref: Some("aira:csu:ctx.basic".into()),
            action: action.into(),
            object_refs: vec![],
            artifact_refs: vec![],
            context_refs: vec![],
            evidence_refs: vec![],
            requested_at: Timestamp::parse("2026-07-10T12:00:00Z").unwrap(),
        }
    }

    fn sig() -> Signature {
        aira_object::local_test_signature(aira_object::LOCAL_TEST_DOMAIN_MSG)
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn decision_schema_and_unknown_deny() {
        let mut log = MemoryEventLog::new();
        let mut gate = PolicyGate::new(sig());
        gate.allow_action("read_public_artifact");

        let allow = gate
            .check(query("read_public_artifact"), Some(&mut log))
            .unwrap();
        assert_eq!(allow.decision, PolicyDecisionKind::Allow);

        let deny = gate
            .check(query("unknown_explode"), Some(&mut log))
            .unwrap();
        assert_eq!(deny.decision, PolicyDecisionKind::Deny);

        gate.require_action("needs_extra_evidence");
        let require = gate
            .check(query("needs_extra_evidence"), Some(&mut log))
            .unwrap();
        assert_eq!(require.decision, PolicyDecisionKind::Require);

        let evaluated = log
            .all()
            .iter()
            .filter(|e| e.event_type == EventType::PolicyEvaluated)
            .count();
        assert_eq!(evaluated, 3);

        let v = serde_json::to_value(&deny).unwrap();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:policy:decision:0.1", &v).unwrap();
    }

    #[test]
    fn policy_event_ids_include_run_nonce() {
        let mut log = MemoryEventLog::new();
        let mut gate = PolicyGate::new(sig()).with_run_nonce("abc");
        gate.allow_action("read_public_artifact");
        gate.check(query("read_public_artifact"), Some(&mut log))
            .unwrap();
        gate.check(query("unknown"), Some(&mut log)).unwrap();
        let ids: Vec<_> = log
            .all()
            .iter()
            .filter(|e| e.event_type == EventType::PolicyEvaluated)
            .map(|e| e.event_id.as_str().to_string())
            .collect();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "aira:event:policyabc_1");
        assert_eq!(ids[1], "aira:event:policyabc_2");
        assert_ne!(ids[0], ids[1]);
    }

    #[test]
    fn two_gates_without_shared_nonce_emit_distinct_policy_ids() {
        let mut log_a = MemoryEventLog::new();
        let mut log_b = MemoryEventLog::new();
        let mut a = PolicyGate::new(sig());
        let mut b = PolicyGate::new(sig());
        a.allow_action("read_public_artifact");
        b.allow_action("read_public_artifact");
        a.check(query("read_public_artifact"), Some(&mut log_a))
            .unwrap();
        b.check(query("read_public_artifact"), Some(&mut log_b))
            .unwrap();
        let id_a = log_a.all()[0].event_id.as_str();
        let id_b = log_b.all()[0].event_id.as_str();
        assert_ne!(id_a, id_b, "default gate nonces must not collide");
        assert!(id_a.starts_with("aira:event:policy"));
        assert!(id_b.starts_with("aira:event:policy"));
    }
}
