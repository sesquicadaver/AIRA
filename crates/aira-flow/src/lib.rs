//! AIRA local operational flow (Issue Set Epic 7 / #47–#56 + Epic 8 local node).
//!
//! Wires Problem submit → basic CSU pipeline → Verified Result / Evidence.
//! Epic 8 adds `.aira` layout persistence via [`local`].

mod local;
mod plane;

pub use local::{
    init_node, load_config, LocalSession, NodeConfig, NodePaths, ProblemRecord, DEFAULT_AIRA_ROOT,
};
pub use plane::{FlowError, OperationalPlane, SubmitOutcome};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_artifact::{ArtifactStore, ArtifactType};
    use aira_core::ObjectStore;
    use aira_csu::support::{json_bytes, make_artifact, make_event};
    use aira_event::EventType;
    use aira_object::AiraRef;
    use serde_json::json;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn submit_creates_problem_and_event_schema_valid() {
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let out = plane.submit_problem("Calculate 2 + 2").unwrap();
        assert!(matches!(out, SubmitOutcome::Completed { .. }));
        let problem = plane.problem_ref().unwrap();
        let desc = plane.objects().get_by_object_id(problem).unwrap().unwrap();
        let v = serde_json::to_value(&desc).unwrap();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:core:object-descriptor:0.1", &v)
            .unwrap();
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::ProblemSubmitted));
    }

    #[test]
    fn calculate_two_plus_two_demo() {
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let out = plane.submit_problem("Calculate 2 + 2").unwrap();
        let SubmitOutcome::Completed {
            verified_artifact_id,
            result,
            ..
        } = out
        else {
            panic!("expected completed flow, got {out:?}");
        };
        assert_eq!(result["result"], json!(4.0));
        assert_eq!(result["verification_status"], json!("VERIFIED"));
        assert_eq!(result["confidence"], json!(1.0));
        assert!(result.get("evidence_refs").is_some());
        assert!(result.get("provenance_refs").is_some());

        let (_d, bytes) = plane.artifacts().resolve(&verified_artifact_id).unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["result"], json!(4.0));

        for required in [
            EventType::ProblemSubmitted,
            EventType::ContextResolved,
            EventType::ReductionCompleted,
            EventType::CapsuleCreated,
            EventType::CapsuleCompleted,
            EventType::VerificationCompleted,
            EventType::ResultPublished,
        ] {
            assert!(
                plane.events().iter().any(|e| e.event_type == required),
                "missing {required:?}"
            );
        }
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::ArtifactPublished
                || e.event_type == EventType::FailureEvidenceCreated
                || plane.artifacts().resolve(&verified_artifact_id).is_ok()));
        // Evidence artifact exists for ResultPublished
        assert!(plane.has_evidence_for_results());
    }

    #[test]
    fn ready_solution_reuse_skips_execution() {
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let ready_payload = json_bytes(&json!({
            "result": 4.0,
            "verification_status": "VERIFIED",
            "confidence": 1.0,
            "scope": { "scope_type": "local", "description": "ready" },
            "evidence_refs": [],
            "provenance_refs": []
        }));
        let ready = make_artifact(
            "aira:artifact:ready42",
            ArtifactType::ReadySolutionArtifact,
            &ready_payload,
            vec![],
        );
        let ready_id = ready.artifact_id.clone();
        plane
            .artifacts_mut()
            .publish(ready, &ready_payload)
            .unwrap();
        plane.enable_ready_solution(ready_id).unwrap();

        let out = plane.submit_problem("Calculate 2 + 2").unwrap();
        assert!(matches!(out, SubmitOutcome::Completed { .. }));
        assert!(!plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::CapsuleCompleted));
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::ResultPublished));
        assert!(plane
            .events()
            .iter()
            .any(|e| e.payload_ref.as_deref() == Some("reuse:ready_solution")));
    }

    #[test]
    fn failure_to_evidence_demo() {
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let missing = AiraRef::parse("aira:artifact:missing99").unwrap();
        let ev = make_event(
            "aira:event:badcap1",
            EventType::CapsuleCreated,
            vec![AiraRef::parse("aira:problem:fail1").unwrap()],
            vec![missing],
            vec![],
            Some("math.eval.safe".into()),
        );
        plane.inject_and_drain(ev).unwrap();
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::CapsuleFailed));
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::FailureEvidenceCreated));
        assert!(!plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::VerificationCompleted));
        assert!(!plane.has_verified_result_artifact());
    }

    #[test]
    fn normative_split_stub_does_not_autocollapse() {
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let out = plane
            .submit_problem("Choose either left-norm OR right-norm")
            .unwrap();
        let SubmitOutcome::NeedsHumanCollapse { field_artifact_id } = out else {
            panic!("expected NeedsHumanCollapse, got {out:?}");
        };
        let (desc, bytes) = plane.artifacts().resolve(&field_artifact_id).unwrap();
        assert_eq!(desc.artifact_type, ArtifactType::OperationalArtifact);
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["requires_human_collapse"], json!(true));
        assert!(!plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::CapsuleCompleted));
    }

    #[test]
    fn local_init_submit_status_and_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        let cfg = load_config(&root).unwrap();
        assert_eq!(cfg.node.profile, "C1");
        assert!(root.join("db/aira.sqlite").exists());
        assert!(root.join("artifacts/sha256").exists());

        let mut session = LocalSession::open(&root).unwrap();
        let out = session.submit_problem("Calculate 2 + 2").unwrap();
        let SubmitOutcome::Completed {
            problem_id,
            verified_artifact_id,
            ..
        } = out
        else {
            panic!("expected completed");
        };

        let status = session.problem_status(problem_id.as_str()).unwrap();
        assert_eq!(status.status, "completed");
        let result = session.get_result(problem_id.as_str()).unwrap();
        assert_eq!(result["result"], json!(4.0));
        let (_desc, bytes) = session.get_artifact(verified_artifact_id.as_str()).unwrap();
        assert!(!bytes.is_empty());
        let tail = session.event_tail(50).unwrap();
        assert!(tail
            .iter()
            .any(|e| e.event_type == EventType::ProblemSubmitted));
    }

    #[test]
    fn local_session_registers_node_identity() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        let sk = ed25519_dalek::SigningKey::from_bytes(&[9u8; 32]);
        let id = "aira:identity:session-demo";
        std::fs::create_dir_all(root.join("identity")).unwrap();
        std::fs::write(
            root.join("identity/local.ed25519"),
            format!("{}\n", hex::encode(sk.to_bytes())),
        )
        .unwrap();
        std::fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "session-demo",
                "public_key": {
                    "algorithm": "ed25519",
                    "key_hex": hex::encode(sk.verifying_key().to_bytes())
                },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();
        let _session = LocalSession::open(&root).unwrap();
        let ring = aira_object::process_keyring_snapshot();
        let msg = b"session-open-registers";
        let sig = ring
            .sign(&aira_object::AiraRef::parse(id).unwrap(), msg)
            .unwrap();
        aira_object::verify_ed25519(&sig, msg).unwrap();
    }
}
