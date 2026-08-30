//! Model acquisition: policy gate, quarantine, verify, activate, share gate + local
//! publish + local capability advertisement (QUEUE #60–#68). Inventory refresh is
//! CLI-orchestrated (no CSU↛CSU). Not wired into C1. `network=none`.

mod activate;
mod error;
mod manifest;
mod policy;
mod publish;
mod quarantine;
mod types;
mod util;
mod verify;

pub use activate::*;
pub use error::*;
pub use manifest::*;
pub use policy::*;
pub use publish::*;
pub use quarantine::*;
pub use types::*;
pub use verify::*;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use aira_csu::CsuType;
    use aira_object::{active_signature, ContentHash};
    use serde_json::{json, Map, Value};

    use super::*;

    fn init_min_root(root: &Path) {
        for d in ["artifacts", "events", "models", "identity"] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        fs::write(
            root.join("events").join("event-log.json"),
            "{\"events\":[]}",
        )
        .unwrap();
        fs::write(
            root.join("config.json"),
            r#"{"node":{"mode":"local","profile":"C1"},"security":{"allow_network_for_csu":false,"allow_shell_for_csu":false,"require_signed_artifacts":true,"require_signed_events":true,"require_signed_csu_manifests":true},"storage":{"object_store":"sqlite","event_log":"json","artifact_store":"filesystem"},"csu":{"autoload":[]}}"#,
        )
        .unwrap();
    }

    fn weight_files(root: &Path) -> Vec<String> {
        fs::read_dir(root.join("models"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                if n.ends_with(".gguf") || n.ends_with(".safetensors") {
                    Some(n)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_network_none_scoped_fs() {
        let m = acquisition_manifest();
        assert_eq!(m.sandbox.network, "none");
        assert_eq!(m.sandbox.filesystem, "scoped");
        assert_eq!(m.csu_type, CsuType::Custom);
    }

    #[test]
    fn deny_without_policy_emits_decision() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = request_download(
            dir.path(),
            "aira:model:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert!(!out.policy_present);
        assert!(out.reason.contains("no acquisition policy"));
        assert!(dir.path().join(DECISION_POINTER_REL).exists());
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        assert!(!events.is_empty());
        let payload = events
            .last()
            .unwrap()
            .get("payload_ref")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(payload.contains("op:policy-denied:download:"));
    }

    #[test]
    fn deny_when_auto_download_false() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), false).unwrap();
        let out = request_download(dir.path(), "aira:model:example").unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert_eq!(out.auto_download, Some(false));
        assert!(out.reason.contains("auto_download=false"));
    }

    #[test]
    fn allow_when_auto_download_true_no_transfer() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let out = request_download(dir.path(), "aira:model:example").unwrap();
        assert_eq!(out.decision, GateDecision::Allow);
        assert_eq!(out.auto_download, Some(true));
        assert_eq!(out.reason_ref, "aira:reason:auto-download-true");
        assert!(out.decision_artifact_id.contains("acq-allow"));
        let pointer: DecisionPointer = serde_json::from_str(
            &fs::read_to_string(dir.path().join(DECISION_POINTER_REL)).unwrap(),
        )
        .unwrap();
        assert_eq!(pointer.decision, "ALLOW");
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        let payload = events
            .last()
            .unwrap()
            .get("payload_ref")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(payload.contains("op:policy-allowed:download:"));
        assert!(weight_files(dir.path()).is_empty());
        assert!(!dir.path().join("models/quarantine").exists());
    }

    #[test]
    fn quarantine_fetch_after_allow_copies_local_source() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("outside-weights.gguf");
        fs::write(&src, b"fake-gguf-bytes").unwrap();
        let out = fetch_to_quarantine(dir.path(), "aira:model:example", &src).unwrap();
        match out {
            FetchOutcome::Quarantined {
                gate,
                quarantine_path,
                bytes,
                ..
            } => {
                assert_eq!(gate.decision, GateDecision::Allow);
                assert_eq!(bytes, 15);
                assert!(Path::new(&quarantine_path).exists());
                assert!(quarantine_path.contains("quarantine"));
                assert!(!quarantine_path.contains("verified"));
            }
            FetchOutcome::Denied(_) => panic!("expected quarantine"),
        }
        assert!(dir.path().join(QUARANTINE_POINTER_REL).exists());
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        let joined: String = events
            .iter()
            .filter_map(|e| e.get("payload_ref").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("op:quarantine-fetched:download:"));
        // Not activated into inventory cache root as loose weight.
        assert!(weight_files(dir.path()).is_empty());
    }

    #[test]
    fn quarantine_denied_without_policy_no_copy() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let src = dir.path().join("x.gguf");
        fs::write(&src, b"data").unwrap();
        let out = fetch_to_quarantine(dir.path(), "aira:model:x", &src).unwrap();
        assert!(matches!(out, FetchOutcome::Denied(_)));
        assert!(!dir.path().join("models/quarantine").exists());
    }

    #[test]
    fn quarantine_rejects_http_source() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let err = fetch_to_quarantine(
            dir.path(),
            "aira:model:x",
            Path::new("https://example.com/m.gguf"),
        )
        .unwrap_err();
        assert!(matches!(err, AcquisitionError::RemoteSource(_)));
    }

    fn signed_model_artifact(model_id: &str, content_hash: &str) -> Value {
        let mut body = Map::new();
        body.insert(
            "payload_schema".into(),
            json!("aira:schema:model:artifact:0.1"),
        );
        body.insert("model_id".into(), json!(model_id));
        body.insert("format".into(), json!("gguf"));
        body.insert("quantization".into(), json!("int4"));
        body.insert("parameter_class".into(), json!("7B"));
        body.insert("content_hash".into(), json!(content_hash));
        body.insert(
            "provenance_refs".into(),
            json!(["aira:identity:local-test"]),
        );
        let for_sign = Value::Object(body.clone());
        let raw = serde_json::to_vec(&for_sign).unwrap();
        let sig = active_signature(&raw).unwrap();
        body.insert("signature".into(), serde_json::to_value(&sig).unwrap());
        Value::Object(body)
    }

    #[test]
    fn verify_promotes_to_verified_on_match() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("ok.gguf");
        fs::write(&src, b"verify-me-please").unwrap();
        fetch_to_quarantine(dir.path(), "aira:model:ok", &src).unwrap();
        let observed = ContentHash::sha256_bytes(b"verify-me-please");
        let art = signed_model_artifact("aira:model:ok", observed.as_str());
        let art_path = dir.path().join("model.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        let out = verify_quarantine(dir.path(), &art_path).unwrap();
        match out {
            VerifyOutcome::Verified {
                verified_path,
                content_hash,
                ..
            } => {
                assert!(Path::new(&verified_path).exists());
                assert!(verified_path.contains("verified"));
                assert_eq!(content_hash, observed.as_str());
                // Quarantine retained.
                assert!(dir.path().join(QUARANTINE_POINTER_REL).exists());
            }
            VerifyOutcome::Rejected { reason, .. } => panic!("unexpected reject: {reason}"),
        }
        assert!(dir.path().join(VERIFIED_POINTER_REL).exists());
    }

    #[test]
    fn verify_rejects_hash_mismatch_keeps_quarantine() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("bad.gguf");
        fs::write(&src, b"actual-bytes").unwrap();
        let fetch = fetch_to_quarantine(dir.path(), "aira:model:bad", &src).unwrap();
        let qpath = match fetch {
            FetchOutcome::Quarantined {
                quarantine_path, ..
            } => quarantine_path,
            FetchOutcome::Denied(_) => panic!("expected quarantine"),
        };
        let wrong = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let art = signed_model_artifact("aira:model:bad", wrong);
        let art_path = dir.path().join("bad.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        let out = verify_quarantine(dir.path(), &art_path).unwrap();
        match out {
            VerifyOutcome::Rejected {
                reason_ref,
                quarantine_path,
                ..
            } => {
                assert_eq!(reason_ref, "aira:reason:model-hash-mismatch");
                assert_eq!(quarantine_path, qpath);
                assert!(Path::new(&qpath).exists());
            }
            VerifyOutcome::Verified { .. } => panic!("expected reject"),
        }
        assert!(!dir.path().join(VERIFIED_POINTER_REL).exists());
    }

    #[test]
    fn verify_rejects_unsigned_testsig() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("u.gguf");
        fs::write(&src, b"unsigned-bytes").unwrap();
        fetch_to_quarantine(dir.path(), "aira:model:u", &src).unwrap();
        let observed = ContentHash::sha256_bytes(b"unsigned-bytes");
        let mut art = signed_model_artifact("aira:model:u", observed.as_str());
        art.as_object_mut().unwrap().insert(
            "signature".into(),
            json!({
                "algorithm": "ed25519",
                "key_ref": "aira:identity:local-test",
                "signature_value": "TESTSIG"
            }),
        );
        let art_path = dir.path().join("u.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        let out = verify_quarantine(dir.path(), &art_path).unwrap();
        match out {
            VerifyOutcome::Rejected { reason_ref, .. } => {
                assert_eq!(reason_ref, "aira:reason:model-unsigned");
            }
            VerifyOutcome::Verified { .. } => panic!("expected unsigned reject"),
        }
    }

    #[test]
    fn activate_copies_verified_to_cache_no_execution() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("act.gguf");
        fs::write(&src, b"activate-bytes").unwrap();
        fetch_to_quarantine(dir.path(), "aira:model:act", &src).unwrap();
        let observed = ContentHash::sha256_bytes(b"activate-bytes");
        let art = signed_model_artifact("aira:model:act", observed.as_str());
        let art_path = dir.path().join("act.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        verify_quarantine(dir.path(), &art_path).unwrap();
        let out = activate_verified(dir.path()).unwrap();
        assert!(Path::new(&out.cache_path).exists());
        assert!(out.cache_path.contains("cache"));
        assert!(dir.path().join(ACTIVATED_POINTER_REL).exists());
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let joined: String = log
            .get("events")
            .and_then(|e| e.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("payload_ref").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("op:model-installed:activate:"));
    }

    #[test]
    fn activate_requires_verified_pointer() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let err = activate_verified(dir.path()).unwrap_err();
        assert!(matches!(err, AcquisitionError::NoVerified));
    }

    #[test]
    fn publish_deny_without_policy() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = request_publish(dir.path(), "aira:model:share-me").unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert!(!out.policy_present);
        assert!(dir.path().join(SHARE_DECISION_POINTER_REL).exists());
    }

    #[test]
    fn publish_deny_when_share_false() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_acquisition_policy(dir.path(), false, false).unwrap();
        let out = request_publish(dir.path(), "aira:model:share-me").unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert_eq!(out.share_custom_models, Some(false));
        assert_eq!(out.reason_ref, "aira:reason:share-custom-models-false");
    }

    #[test]
    fn publish_allow_when_share_true_no_offer_bytes() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_acquisition_policy(dir.path(), false, true).unwrap();
        let out = request_publish(dir.path(), "aira:model:share-me").unwrap();
        assert_eq!(out.decision, GateDecision::Allow);
        assert_eq!(out.share_custom_models, Some(true));
        assert!(out.decision_artifact_id.contains("share-allow"));
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let payload = log
            .get("events")
            .and_then(|e| e.as_array())
            .unwrap()
            .last()
            .unwrap()
            .get("payload_ref")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(payload.contains("op:policy-allowed:publish:"));
        // Gate alone still does not write ShareOffer pointer.
        assert!(!dir.path().join(SHARE_OFFER_POINTER_REL).exists());
    }

    #[test]
    fn publish_local_deny_without_policy() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = publish_local(dir.path(), "aira:model:x", "local", false).unwrap();
        assert!(matches!(out, PublishOutcome::Denied(_)));
        assert!(!dir.path().join(SHARE_OFFER_POINTER_REL).exists());
    }

    #[test]
    fn publish_local_requires_activated_cache() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_acquisition_policy(dir.path(), false, true).unwrap();
        let err = publish_local(dir.path(), "aira:model:share-me", "local", false).unwrap_err();
        assert!(matches!(err, AcquisitionError::NoActivated));
    }

    #[test]
    fn publish_local_rejects_bad_visibility() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let err = publish_local(dir.path(), "aira:model:x", "global", false).unwrap_err();
        assert!(matches!(err, AcquisitionError::BadVisibility(_)));
    }

    #[test]
    fn publish_local_writes_signed_descriptors_from_cache() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_acquisition_policy(dir.path(), true, true).unwrap();
        let src = dir.path().join("pub.gguf");
        fs::write(&src, b"publish-local-bytes").unwrap();
        fetch_to_quarantine(dir.path(), "aira:model:pub", &src).unwrap();
        let observed = ContentHash::sha256_bytes(b"publish-local-bytes");
        let art = signed_model_artifact("aira:model:pub", observed.as_str());
        let art_path = dir.path().join("pub.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        verify_quarantine(dir.path(), &art_path).unwrap();
        activate_verified(dir.path()).unwrap();

        let out = publish_local(dir.path(), "aira:model:pub", "opt_in", false).unwrap();
        match out {
            PublishOutcome::Published {
                model_artifact_id,
                share_offer_artifact_id,
                offer_id,
                capability_artifact_id,
                capability_id,
                visibility,
                content_hash,
                ..
            } => {
                assert!(model_artifact_id.contains("model-desc"));
                assert!(share_offer_artifact_id.contains("share-offer"));
                assert!(capability_artifact_id.contains("capability-ad"));
                assert!(capability_id.starts_with("aira:capability:model.share:"));
                assert!(offer_id.starts_with("aira:share:"));
                assert_eq!(visibility, "opt_in");
                assert_eq!(content_hash, observed.as_str());
            }
            PublishOutcome::Denied(_) => panic!("expected Published"),
        }
        assert!(dir.path().join(SHARE_OFFER_POINTER_REL).exists());
        assert!(dir.path().join(CAPABILITY_AD_POINTER_REL).exists());
        let cap_ptr: CapabilityAdPointer = serde_json::from_str(
            &fs::read_to_string(dir.path().join(CAPABILITY_AD_POINTER_REL)).unwrap(),
        )
        .unwrap();
        assert_eq!(cap_ptr.scope_type, "local");
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let joined: String = log
            .get("events")
            .and_then(|e| e.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("payload_ref").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("op:share-published:publish:aira:model:pub:opt_in"));
        assert!(joined.contains("op:capability-advertised:share:aira:model:pub:local"));
    }

    #[test]
    fn fail_closed_audit_download_and_publish_without_allow() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let model_ref = "aira:model:audit-fail-closed";

        let dl = request_download(dir.path(), model_ref).unwrap();
        assert_eq!(dl.decision, GateDecision::Deny);
        assert_eq!(dl.reason_ref, "aira:reason:no-acquisition-policy");
        assert!(dir.path().join(DECISION_POINTER_REL).exists());

        let pub_gate = request_publish(dir.path(), model_ref).unwrap();
        assert_eq!(pub_gate.decision, GateDecision::Deny);
        assert_eq!(pub_gate.reason_ref, "aira:reason:no-acquisition-policy");
        assert!(dir.path().join(SHARE_DECISION_POINTER_REL).exists());

        let src = dir.path().join("weights.gguf");
        fs::write(&src, b"blocked-by-policy").unwrap();
        assert!(matches!(
            fetch_to_quarantine(dir.path(), model_ref, &src),
            Ok(FetchOutcome::Denied(_))
        ));
        assert!(matches!(
            publish_local(dir.path(), model_ref, "local", false),
            Ok(PublishOutcome::Denied(_))
        ));
        assert!(!dir.path().join("models/quarantine").exists());
        assert!(!dir.path().join(SHARE_OFFER_POINTER_REL).exists());

        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let payloads: String = log
            .get("events")
            .and_then(|e| e.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("payload_ref").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        assert!(payloads.contains("op:policy-denied:download:"));
        assert!(payloads.contains("op:policy-denied:publish:"));
    }

    #[test]
    fn publish_local_deny_skips_capability_ad() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = publish_local(dir.path(), "aira:model:x", "local", false).unwrap();
        assert!(matches!(out, PublishOutcome::Denied(_)));
        assert!(!dir.path().join(CAPABILITY_AD_POINTER_REL).exists());
    }
}
