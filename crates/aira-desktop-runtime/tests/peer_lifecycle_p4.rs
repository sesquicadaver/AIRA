//! P4 gossip peer lifecycle (QUEUE #101).

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use aira_desktop_runtime::{
    load_or_create_settings, start, stop, write_settings, DesktopPaths, LifecycleStatus,
    NetworkProfile,
};
use aira_object::{AiraRef, ContentHash, Keyring};
use aira_peer::{gossip_forward_trust_delta, AddressBook, TrustDelta, TRUST_DELTA_MESSAGE_TYPE};
use aira_protocol::{ProtocolEnvelope, ProtocolId, ScopeDescriptor};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn node_bin() -> PathBuf {
    if let Ok(p) = std::env::var("AIRA_NODE_BIN") {
        return PathBuf::from(p);
    }
    for rel in [
        "../../target/debug/aira-node",
        "../../target/release/aira-node",
    ] {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push(rel);
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from("aira-node")
}

fn aira_bin() -> PathBuf {
    if let Ok(p) = std::env::var("AIRA_BIN") {
        return PathBuf::from(p);
    }
    let node = node_bin();
    if let Some(dir) = node.parent() {
        let cand = dir.join("aira");
        if cand.is_file() {
            return cand;
        }
    }
    for profile in ["debug", "release"] {
        let cand = workspace_root().join("target").join(profile).join("aira");
        if cand.is_file() {
            return cand;
        }
    }
    PathBuf::from("aira")
}

fn ensure_bins() -> (PathBuf, PathBuf) {
    let mut node = node_bin();
    let mut aira = aira_bin();
    if node.is_file() && aira.is_file() {
        return (node, aira);
    }
    let status = Command::new(env!("CARGO"))
        .args(["build", "-p", "aira-cli", "-p", "aira-node", "--quiet"])
        .current_dir(workspace_root())
        .status()
        .expect("cargo build aira-cli/aira-node");
    assert!(
        status.success(),
        "cargo build -p aira-cli -p aira-node failed"
    );
    node = node_bin();
    aira = aira_bin();
    assert!(node.is_file(), "missing aira-node ({})", node.display());
    assert!(aira.is_file(), "missing aira ({})", aira.display());
    (node, aira)
}

fn free_listen() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("127.0.0.1:{port}")
}

/// Hostile trust-delta envelope (subject ≠ issuer) for forward-filter smoke (Analyze-53).
fn craft_hostile_trust_delta_envelope(
    root: &std::path::Path,
    delta: &TrustDelta,
) -> ProtocolEnvelope {
    use rand::rngs::OsRng;
    use rand::RngCore;
    delta.validate_shape().unwrap();
    let (local_id, ring) = Keyring::load_node_identity(root).unwrap();
    let json = String::from_utf8(delta.canonical_bytes().unwrap()).unwrap();
    let hash = ContentHash::sha256_bytes(json.as_bytes());
    let signature = ring.sign(&local_id, hash.as_str().as_bytes()).unwrap();
    let mut nonce = [0u8; 8];
    OsRng.fill_bytes(&mut nonce);
    let message_id = AiraRef::parse(format!(
        "aira:message:trust-delta-hostile-{}",
        hex::encode(nonce)
    ))
    .unwrap();
    let created = aira_object::utc_now_rfc3339().unwrap();
    ProtocolEnvelope {
        protocol_id: ProtocolId::Identity,
        protocol_version: "0.1".into(),
        message_type: TRUST_DELTA_MESSAGE_TYPE.into(),
        message_id,
        correlation_id: None,
        causal_refs: vec![],
        issuer_identity: local_id,
        target_scope: ScopeDescriptor::local("peer-trust-delta"),
        policy_refs: vec![],
        payload_hash: hash,
        payload_ref: Some(json),
        created_at: aira_object::Timestamp::parse(created).unwrap(),
        expires_at: None,
        signature,
    }
}

#[test]
fn p4_starts_http_and_gossip_peer() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();
    let http = free_listen();
    let peer = free_listen();
    let mut settings = load_or_create_settings(&paths).unwrap();
    settings.http_listen = http.clone();
    settings.network_profile = NetworkProfile::P4;
    settings.peer_listen = Some(peer.clone());
    write_settings(&paths, &settings).unwrap();

    let outcome = start(&paths, Some(node)).expect("start P4");
    assert_eq!(outcome.status, LifecycleStatus::Running);
    assert_eq!(outcome.peer_listen.as_deref(), Some(peer.as_str()));
    assert!(outcome.peer_pid.is_some());

    let pid_text = std::fs::read_to_string(paths.peer_pid_file()).unwrap();
    assert!(
        pid_text.contains("\"network_profile\": \"P4\""),
        "{pid_text}"
    );
    assert!(
        !pid_text.contains("relay_ttl_days"),
        "P4 pid should not store relay TTL: {pid_text}"
    );

    let _ = stop(&paths);
}

#[tokio::test]
async fn p4_gossip_forward_filter_smoke() {
    let (node, aira) = ensure_bins();
    std::env::set_var("AIRA_BIN", &aira);

    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    paths.ensure_dirs().unwrap();
    let mut settings = load_or_create_settings(&paths).unwrap();
    settings.http_listen = free_listen();
    settings.network_profile = NetworkProfile::P4;
    settings.peer_listen = Some(free_listen());
    write_settings(&paths, &settings).unwrap();

    start(&paths, Some(node)).expect("start P4 gossip peer");
    std::thread::sleep(Duration::from_millis(400));

    let mut book = AddressBook::default();
    book.upsert("aira:identity:would-dial", "127.0.0.1:1");
    book.save(&paths.data_root).unwrap();

    let victim = "aira:identity:gossip-victim-desktop-101";
    let delta = TrustDelta::revoke(victim, Some("hostile-crl".into()));
    let env = craft_hostile_trust_delta_envelope(&paths.data_root, &delta);
    assert_ne!(delta.subject_id, env.issuer_identity.as_str());

    let results = gossip_forward_trust_delta(&paths.data_root, &env, "aira:identity:upstream")
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].skipped);
    assert_eq!(
        results[0].error.as_deref(),
        Some("non-self-sovereign trust-delta")
    );

    let _ = stop(&paths);
}
