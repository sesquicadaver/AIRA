//! PeerInvite QR PNG roundtrip (QUEUE #84).

use aira_desktop_runtime::{
    decode_invite_png, encode_invite_png, export_invite_qr_png, import_invite_qr_file,
    load_or_create_settings, write_settings, DesktopPaths, NetworkProfile, PEER_INVITE_SCHEMA_ID,
};
use aira_object::TrustStore;
use aira_peer::AddressBook;

#[test]
fn qr_png_export_import_trust_and_book() {
    let tmp = tempfile::tempdir().unwrap();
    let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
    let bob = DesktopPaths::for_data_root(tmp.path().join("bob"));
    alice.ensure_dirs().unwrap();
    bob.ensure_dirs().unwrap();

    let mut settings = load_or_create_settings(&alice).unwrap();
    settings.network_profile = NetworkProfile::P1;
    settings.peer_listen = Some("127.0.0.1:19002".into());
    write_settings(&alice, &settings).unwrap();

    let out = tmp.path().join("alice.invite.png");
    let invite = export_invite_qr_png(&alice, &out, None).expect("export qr");
    assert_eq!(invite.payload_schema, PEER_INVITE_SCHEMA_ID);
    assert_eq!(invite.addr.as_deref(), Some("127.0.0.1:19002"));
    assert!(out.is_file());

    let decoded = decode_invite_png(&out).expect("decode");
    assert_eq!(decoded.identity_ref, invite.identity_ref);
    assert_eq!(decoded.public_key_hex, invite.public_key_hex);
    assert_eq!(decoded.addr, invite.addr);

    let applied = import_invite_qr_file(&bob, &out).expect("import qr");
    assert!(applied.trusted);
    assert!(applied.book_updated);

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
        .any(|p| p.identity_id == invite.identity_ref && p.addr == "127.0.0.1:19002"));
}

#[test]
fn encode_decode_roundtrip_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
    let out = tmp.path().join("roundtrip.png");
    let invite = export_invite_qr_png(&alice, &out, Some("127.0.0.1:19111".into())).unwrap();
    encode_invite_png(&invite, &out).unwrap();
    let again = decode_invite_png(&out).unwrap();
    assert_eq!(again, invite);
}

#[test]
fn reject_non_image_file() {
    let tmp = tempfile::tempdir().unwrap();
    let png = tmp.path().join("blank.png");
    std::fs::write(&png, b"not-a-png").unwrap();
    let err = decode_invite_png(&png).unwrap_err().to_string();
    assert!(
        err.contains("open image") || err.contains("no QR") || err.contains("QR"),
        "{err}"
    );
}
