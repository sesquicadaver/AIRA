//! Desktop federation join from descriptor file (QUEUE #103).

use aira_desktop_runtime::{
    join_federation_descriptor_file, read_federation_membership, DesktopPaths,
};
use aira_object::{AiraRef, TrustStore};
use aira_protocol::{
    descriptor_canonical_bytes, membership_path, FederationDescriptor, FEDERATION_DESCRIPTOR_DOMAIN,
};
use ed25519_dalek::SigningKey;

fn signed_descriptor(id: &str, fed: &str, seed: u8) -> FederationDescriptor {
    let sk = SigningKey::from_bytes(&[seed; 32]);
    let pk = hex::encode(sk.verifying_key().to_bytes());
    let mut d = FederationDescriptor {
        schema: FEDERATION_DESCRIPTOR_DOMAIN.into(),
        federation_id: fed.into(),
        federation_type: "private".into(),
        identity_ref: id.into(),
        public_key_hex: pk,
        signature: aira_object::Signature {
            algorithm: "ed25519".into(),
            key_ref: AiraRef::parse(id).unwrap(),
            signature_value: String::new(),
        },
    };
    d.signature = aira_object::sign_with_key(
        AiraRef::parse(id).unwrap(),
        &sk,
        &descriptor_canonical_bytes(&d),
    );
    d
}

#[test]
fn join_descriptor_file_pins_trust_and_membership() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let desc = signed_descriptor(
        "aira:identity:fed-desktop-103",
        "aira:federation:desktop-103",
        42,
    );
    let desc_path = tmp.path().join("fed.json");
    std::fs::write(&desc_path, serde_json::to_string_pretty(&desc).unwrap()).unwrap();

    let out = join_federation_descriptor_file(&paths, &desc_path).unwrap();
    assert!(!out.already_member);
    assert_eq!(out.membership.federation_id, "aira:federation:desktop-103");

    let store = TrustStore::load(&paths.data_root).unwrap();
    assert!(store
        .entries
        .iter()
        .any(|e| e.identity_id == "aira:identity:fed-desktop-103"));
    assert!(membership_path(&paths.data_root).is_file());

    let read = read_federation_membership(&paths).unwrap().unwrap();
    assert_eq!(read.federation_id, out.membership.federation_id);
    assert_eq!(read.identity_ref, out.membership.identity_ref);

    let again = join_federation_descriptor_file(&paths, &desc_path).unwrap();
    assert!(again.already_member);
}

#[test]
fn leave_clears_membership() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let desc = signed_descriptor(
        "aira:identity:fed-desktop-leave",
        "aira:federation:desktop-leave",
        43,
    );
    let desc_path = tmp.path().join("fed.json");
    std::fs::write(&desc_path, serde_json::to_string_pretty(&desc).unwrap()).unwrap();
    join_federation_descriptor_file(&paths, &desc_path).unwrap();
    assert!(read_federation_membership(&paths).unwrap().is_some());

    let out = aira_desktop_runtime::leave_federation_local(&paths).unwrap();
    assert!(out.was_member);
    assert_eq!(
        out.federation_id.as_deref(),
        Some("aira:federation:desktop-leave")
    );
    assert!(read_federation_membership(&paths).unwrap().is_none());
    assert!(!membership_path(&paths.data_root).exists());

    let store = TrustStore::load(&paths.data_root).unwrap();
    assert!(store
        .entries
        .iter()
        .any(|e| e.identity_id == "aira:identity:fed-desktop-leave"));
}

#[test]
fn join_invalid_descriptor_fails_closed() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = DesktopPaths::for_data_root(tmp.path());
    let bad = tmp.path().join("bad.json");
    std::fs::write(&bad, "{\"schema\":\"wrong\"}").unwrap();
    assert!(join_federation_descriptor_file(&paths, &bad).is_err());
    assert!(read_federation_membership(&paths).unwrap().is_none());
}
