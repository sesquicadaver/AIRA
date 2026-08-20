//! PeerInvite export/import roundtrip (QUEUE #83).

use std::path::PathBuf;

use aira_desktop_runtime::{
    export_invite_file, import_invite_file, load_or_create_settings, write_settings, DesktopPaths,
    NetworkProfile, PEER_INVITE_SCHEMA_ID,
};
use aira_object::TrustStore;
use aira_peer::AddressBook;

#[test]
fn export_import_trust_and_book() {
    let tmp = tempfile::tempdir().unwrap();
    let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
    let bob = DesktopPaths::for_data_root(tmp.path().join("bob"));
    alice.ensure_dirs().unwrap();
    bob.ensure_dirs().unwrap();

    let mut settings = load_or_create_settings(&alice).unwrap();
    settings.network_profile = NetworkProfile::P1;
    settings.peer_listen = Some("127.0.0.1:19001".into());
    write_settings(&alice, &settings).unwrap();

    let out = tmp.path().join("alice.invite.json");
    let invite = export_invite_file(&alice, &out, None).expect("export");
    assert_eq!(invite.payload_schema, PEER_INVITE_SCHEMA_ID);
    assert_eq!(invite.addr.as_deref(), Some("127.0.0.1:19001"));
    assert!(out.is_file());

    let applied = import_invite_file(&bob, &out).expect("import");
    assert!(applied.trusted);
    assert!(applied.book_updated);
    assert_eq!(applied.addr.as_deref(), Some("127.0.0.1:19001"));

    let store = TrustStore::load(&bob.data_root).unwrap();
    assert!(store
        .entries
        .iter()
        .any(|e| e.identity_id == invite.identity_ref
            && e.public_key_hex
                .eq_ignore_ascii_case(invite.public_key_hex.trim())));

    let book = AddressBook::load(&bob.data_root).unwrap();
    assert!(book
        .peers
        .iter()
        .any(|p| p.identity_id == invite.identity_ref && p.addr == "127.0.0.1:19001"));
}

#[test]
fn trust_only_skips_book() {
    let tmp = tempfile::tempdir().unwrap();
    let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
    let bob = DesktopPaths::for_data_root(tmp.path().join("bob"));
    let out = tmp.path().join("trust-only.json");
    let invite = export_invite_file(&alice, &out, None).expect("export P0");
    assert!(invite.addr.is_none());

    let applied = import_invite_file(&bob, &out).expect("import");
    assert!(applied.trusted);
    assert!(!applied.book_updated);
    assert!(AddressBook::load(&bob.data_root).unwrap().peers.is_empty());
}

#[test]
fn reject_bad_pubkey() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let bad = tmp.path().join("bad.json");
    std::fs::write(
        &bad,
        r#"{
  "payload_schema": "aira:schema:desktop:peer-invite:0.1",
  "identity_ref": "aira:identity:x",
  "public_key_hex": "deadbeef"
}
"#,
    )
    .unwrap();
    let err = import_invite_file(&paths, &bad).unwrap_err().to_string();
    assert!(
        err.contains("public_key_hex") || err.contains("64"),
        "{err}"
    );
    let _ = PathBuf::from("keep");
}
