//! Two-process ab ovo CI harness binary (QUEUE #249 / Phase N-fix).
//!
//! Roles:
//! - `publisher`: listen → publish Presence to shared local-file ledger → wait peer trust → accept
//! - `discoverer`: empty AddressBook → query ledger → DISCOVERED → trust → promote → dial
//!
//! Orchestrated by `tests/ab_ovo_two_process.rs` via two OS processes.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use aira_flow::NodePaths;
use aira_object::{ensure_trust_defaults, sign_with_key, AiraRef, Keyring};
use aira_peer::{
    accept, admit_peer_trust, dial, discover_admit_promote, empty_capabilities_hash,
    listen_available_loopback, AddressBook, LocalFileRendezvousProvider, NodePresenceRecord,
    PresenceDirectEndpoint, PresenceDraft, PresenceReachability, RendezvousProvider,
};
use ed25519_dalek::SigningKey;

fn usage() -> ! {
    eprintln!(
        "usage:\n  aira-ab-ovo-harness publisher --root DIR --ledger DIR --ready FILE --peer-trust FILE\n  aira-ab-ovo-harness discoverer --root DIR --ledger DIR --ready FILE --peer-trust FILE --as-of RFC3339"
    );
    std::process::exit(2);
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn ensure_identity(root: &Path, name: &str, seed: [u8; 32]) -> (AiraRef, String) {
    let paths = NodePaths::new(root);
    if paths.identity_json().is_file() && paths.identity_key().is_file() {
        let (id, _ring): (AiraRef, Keyring) =
            Keyring::load_node_identity(root).expect("load identity");
        let desc: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(paths.identity_json()).unwrap()).unwrap();
        let pk = desc["public_key"]["key_hex"]
            .as_str()
            .expect("key_hex")
            .to_string();
        let _ = ensure_trust_defaults(root).unwrap();
        return (id, pk);
    }
    fs::create_dir_all(paths.identity_dir()).unwrap();
    let sk = SigningKey::from_bytes(&seed);
    let pub_hex = hex::encode(sk.verifying_key().to_bytes());
    let id = format!("aira:identity:{name}");
    let id_ref = AiraRef::parse(&id).unwrap();
    fs::write(
        paths.identity_key(),
        format!("{}\n", hex::encode(sk.to_bytes())),
    )
    .unwrap();
    let sig = sign_with_key(id_ref.clone(), &sk, id.as_bytes());
    let desc = serde_json::json!({
        "identity_id": id,
        "identity_type": "local",
        "display_name": name,
        "public_key": { "algorithm": "ed25519", "key_hex": pub_hex },
        "created_at": "2026-09-06T00:00:00Z",
        "key_path": "identity/local.ed25519",
        "signature": sig
    });
    fs::write(
        paths.identity_json(),
        serde_json::to_string_pretty(&desc).unwrap(),
    )
    .unwrap();
    let _ = ensure_trust_defaults(root).unwrap();
    (id_ref, pub_hex)
}

fn wait_file(path: &Path, timeout: Duration) -> String {
    let start = Instant::now();
    loop {
        if path.is_file() {
            return fs::read_to_string(path).expect("read ready/trust file");
        }
        if start.elapsed() > timeout {
            panic!("timeout waiting for {}", path.display());
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn write_atomic(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, body).unwrap();
    fs::rename(&tmp, path).unwrap();
}

fn signed_presence(
    peer_root: &Path,
    id: &AiraRef,
    pk: &str,
    host: &str,
    port: u16,
) -> NodePresenceRecord {
    NodePresenceRecord::draft(PresenceDraft {
        identity_ref: id.as_str().into(),
        identity_public_key: pk.into(),
        sequence: 1,
        created_at: "2026-09-05T12:00:00Z".into(),
        expires_at: "2026-09-12T12:00:00Z".into(),
        direct_endpoints: vec![PresenceDirectEndpoint {
            transport: "tcp-peer".into(),
            host: host.into(),
            port,
            reachability_state: PresenceReachability::Unknown,
            observed_at: "2026-09-05T12:00:00Z".into(),
        }],
        relay_endpoints: vec![],
        capabilities_hash: empty_capabilities_hash(),
    })
    .unwrap()
    .sign_for_node_root(peer_root)
    .unwrap()
}

async fn run_publisher(
    root: PathBuf,
    ledger: PathBuf,
    ready: PathBuf,
    peer_trust: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let (ida, pka) = ensure_identity(&root, "harness-a", [91u8; 32]);
    let (listener, addr) = listen_available_loopback().await?;
    let port = addr.port();
    let mut provider = LocalFileRendezvousProvider::open(&ledger)?;
    let presence = signed_presence(&root, &ida, &pka, "127.0.0.1", port);
    provider.publish_presence(presence)?;
    write_atomic(
        &ready,
        &format!("127.0.0.1:{port}\n{}\n{}\n", ida.as_str(), pka),
    );
    eprintln!("ab-ovo-harness publisher ready {}", addr);

    let trust_body = wait_file(&peer_trust, Duration::from_secs(30));
    let mut lines = trust_body.lines();
    let peer_id = lines.next().ok_or("peer trust missing id")?;
    let peer_pk = lines.next().ok_or("peer trust missing pk")?;
    admit_peer_trust(&root, peer_id, peer_pk)?;

    let server = accept(&listener, &root).await?;
    println!("ab-ovo-harness: accepted {}", server.peer_id.as_str());
    Ok(())
}

async fn run_discoverer(
    root: PathBuf,
    ledger: PathBuf,
    ready: PathBuf,
    peer_trust: PathBuf,
    as_of: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (idb, pkb) = ensure_identity(&root, "harness-b", [92u8; 32]);
    assert!(
        AddressBook::load(&root)?.peers.is_empty(),
        "discoverer AddressBook must start empty"
    );

    let ready_body = wait_file(&ready, Duration::from_secs(30));
    let mut rlines = ready_body.lines();
    let _addr = rlines.next().ok_or("ready missing addr")?;
    let peer_a = rlines.next().ok_or("ready missing peer id")?.to_string();
    let peer_a_pk = rlines.next().ok_or("ready missing peer pk")?.to_string();

    // Poll shared ledger until presence is visible (independent process open).
    let start = Instant::now();
    let presence_seen = loop {
        let provider = LocalFileRendezvousProvider::open(&ledger)?;
        if provider.query_identity(&peer_a)?.is_some() {
            break true;
        }
        if start.elapsed() > Duration::from_secs(15) {
            break false;
        }
        thread::sleep(Duration::from_millis(50));
    };
    assert!(presence_seen, "presence not visible in shared ledger");

    let provider = LocalFileRendezvousProvider::open(&ledger)?;
    let pre = discover_admit_promote(&root, &provider, &peer_a, &as_of, true);
    assert!(
        matches!(pre, Err(aira_peer::PeerError::Untrusted(_))),
        "must fail-closed before trust: {pre:?}"
    );
    assert!(
        AddressBook::load(&root)?.peers.is_empty(),
        "book must stay empty without trust"
    );

    admit_peer_trust(&root, &peer_a, &peer_a_pk)?;
    write_atomic(&peer_trust, &format!("{}\n{}\n", idb.as_str(), pkb));

    let provider = LocalFileRendezvousProvider::open(&ledger)?;
    let (_pres, ep) = discover_admit_promote(&root, &provider, &peer_a, &as_of, false)?;
    assert_eq!(ep.identity_id, peer_a);

    let client = dial(&root, &peer_a).await?;
    println!(
        "ab-ovo-harness: dialed {} as {}",
        client.peer_id.as_str(),
        idb.as_str()
    );
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }
    let role = args[1].as_str();
    let root = PathBuf::from(arg_value(&args, "--root").unwrap_or_else(|| usage()));
    let ledger = PathBuf::from(arg_value(&args, "--ledger").unwrap_or_else(|| usage()));
    let ready = PathBuf::from(arg_value(&args, "--ready").unwrap_or_else(|| usage()));
    let peer_trust = PathBuf::from(arg_value(&args, "--peer-trust").unwrap_or_else(|| usage()));

    let result = match role {
        "publisher" => run_publisher(root, ledger, ready, peer_trust).await,
        "discoverer" => {
            let as_of =
                arg_value(&args, "--as-of").unwrap_or_else(|| "2026-09-06T12:00:00Z".into());
            run_discoverer(root, ledger, ready, peer_trust, as_of).await
        }
        _ => usage(),
    };
    if let Err(e) = result {
        eprintln!("ab-ovo-harness error: {e}");
        std::process::exit(1);
    }
}
