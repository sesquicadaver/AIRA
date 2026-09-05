//! Testable Desktop GUI actions (QUEUE #85) — no egui dependency.

use std::path::Path;

use anyhow::Result;

use aira_desktop_runtime::{
    build_local_invite, encode_invite_rgba, ensure_bootstrap, export_invite_file,
    export_invite_qr_png, import_invite_file, import_invite_qr_file, import_invite_qr_luma,
    join_federation_descriptor_file, normalize_settings, read_federation_membership,
    run_discv_announce, run_discv_find, run_stun_query, submit_desktop_problem, write_settings,
    DesktopPaths, DesktopSettings, DiscoveryStunOutcome, ImportInviteOutcome, NetworkProfile,
    PeerInvite, DEFAULT_PEER_LISTEN, DEFAULT_RELAY_TTL_DAYS,
};
use aira_peer::DiscvFindReport;
use aira_protocol::{FederationMembership, JoinOutcome};

use crate::work_view::{format_work_result, WorkResultView};

/// Apply supported profile; fill defaults for `peer_listen` / `relay_ttl_days` on P1–P4.
pub fn apply_network_profile(
    settings: &mut DesktopSettings,
    profile: NetworkProfile,
    peer_listen_edit: &str,
    relay_ttl_days: Option<u32>,
) -> Result<()> {
    if !profile.is_supported() {
        anyhow::bail!("unsupported network_profile {profile:?}");
    }
    settings.network_profile = profile;
    match profile {
        NetworkProfile::P1 | NetworkProfile::P2 | NetworkProfile::P3 | NetworkProfile::P4 => {
            let trimmed = peer_listen_edit.trim();
            settings.peer_listen = Some(if trimmed.is_empty() {
                DEFAULT_PEER_LISTEN.to_string()
            } else {
                trimmed.to_string()
            });
        }
        NetworkProfile::P0 => {
            settings.peer_listen = None;
        }
        _ => unreachable!("is_supported"),
    }
    if profile.is_relay_profile() {
        settings.relay_ttl_days = Some(relay_ttl_days.unwrap_or(DEFAULT_RELAY_TTL_DAYS));
    }
    normalize_settings(settings)
}

/// Persist settings after profile/listen edits.
pub fn persist_settings(paths: &DesktopPaths, settings: &DesktopSettings) -> Result<()> {
    write_settings(paths, settings)
}

/// Join federation from a signed descriptor JSON file (P5 wizard backend).
pub fn join_federation_descriptor(
    paths: &DesktopPaths,
    descriptor_path: &Path,
) -> Result<JoinOutcome> {
    join_federation_descriptor_file(paths, descriptor_path)
}

/// Read local federation membership for status display.
pub fn federation_membership(paths: &DesktopPaths) -> Result<Option<FederationMembership>> {
    read_federation_membership(paths)
}

/// Explicit STUN Binding query (P6; no public default server).
pub fn discovery_stun_query(
    paths: &DesktopPaths,
    stun_server: &str,
) -> Result<DiscoveryStunOutcome> {
    run_stun_query(paths, stun_server)
}

/// UDP discv announce shortcut (explicit addr required).
pub fn discovery_discv_announce(
    paths: &DesktopPaths,
    to: &str,
    advertised_addr: &str,
) -> Result<String> {
    run_discv_announce(paths, to, advertised_addr)
}

/// Iterative discv FIND shortcut.
pub fn discovery_discv_find(
    paths: &DesktopPaths,
    key_ref: &str,
    to: Option<&str>,
    k: u32,
) -> Result<DiscvFindReport> {
    run_discv_find(paths, key_ref, to, k)
}

/// Build local invite + RGBA QR preview for the GUI (bootstraps identity if needed).
pub fn preview_invite_qr(
    paths: &DesktopPaths,
    settings: &mut DesktopSettings,
) -> Result<(PeerInvite, usize, usize, Vec<u8>)> {
    paths.ensure_dirs()?;
    ensure_bootstrap(paths, settings)?;
    let invite = build_local_invite(paths, settings, None)?;
    let (w, h, rgba) = encode_invite_rgba(&invite)?;
    Ok((invite, w, h, rgba))
}

/// Export invite JSON to `out`.
pub fn export_json(paths: &DesktopPaths, out: &Path, addr: Option<String>) -> Result<PeerInvite> {
    export_invite_file(paths, out, addr)
}

/// Export invite QR PNG to `out`.
pub fn export_qr(paths: &DesktopPaths, out: &Path, addr: Option<String>) -> Result<PeerInvite> {
    export_invite_qr_png(paths, out, addr)
}

/// Import invite JSON file.
pub fn import_json(paths: &DesktopPaths, file: &Path) -> Result<ImportInviteOutcome> {
    import_invite_file(paths, file)
}

/// Import invite QR PNG/image file.
pub fn import_qr(paths: &DesktopPaths, file: &Path) -> Result<ImportInviteOutcome> {
    import_invite_qr_file(paths, file)
}

/// Import invite from a camera / in-memory luma QR frame.
pub fn import_qr_luma(paths: &DesktopPaths, img: image::GrayImage) -> Result<ImportInviteOutcome> {
    import_invite_qr_luma(paths, img)
}

/// Submit problem text to the supervised node (`POST /v1/problems`).
///
/// Returns a human-first Work view (`result.result` + status + verification),
/// not a raw VRA dump.
pub fn submit_problem(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
    text: &str,
) -> Result<WorkResultView> {
    let v = submit_desktop_problem(paths, settings, text)?;
    Ok(format_work_result(&v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_desktop_runtime::{load_or_create_settings, PEER_INVITE_SCHEMA_ID};
    use aira_object::TrustStore;
    use aira_peer::AddressBook;

    #[test]
    fn p2_profile_persist_peer_listen() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let mut settings = load_or_create_settings(&paths).unwrap();
        apply_network_profile(&mut settings, NetworkProfile::P2, "127.0.0.1:49199", None).unwrap();
        persist_settings(&paths, &settings).unwrap();
        let loaded = load_or_create_settings(&paths).unwrap();
        assert_eq!(loaded.network_profile, NetworkProfile::P2);
        assert_eq!(loaded.peer_listen.as_deref(), Some("127.0.0.1:49199"));
    }

    #[test]
    fn p3_profile_persist_relay_ttl() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let mut settings = load_or_create_settings(&paths).unwrap();
        apply_network_profile(
            &mut settings,
            NetworkProfile::P3,
            "127.0.0.1:49201",
            Some(21),
        )
        .unwrap();
        persist_settings(&paths, &settings).unwrap();
        let loaded = load_or_create_settings(&paths).unwrap();
        assert_eq!(loaded.network_profile, NetworkProfile::P3);
        assert_eq!(loaded.peer_listen.as_deref(), Some("127.0.0.1:49201"));
        assert_eq!(loaded.relay_ttl_days, Some(21));
    }

    #[test]
    fn p4_profile_persist_peer_listen() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let mut settings = load_or_create_settings(&paths).unwrap();
        apply_network_profile(&mut settings, NetworkProfile::P4, "127.0.0.1:49207", None).unwrap();
        persist_settings(&paths, &settings).unwrap();
        let loaded = load_or_create_settings(&paths).unwrap();
        assert_eq!(loaded.network_profile, NetworkProfile::P4);
        assert_eq!(loaded.peer_listen.as_deref(), Some("127.0.0.1:49207"));
        assert!(loaded.relay_ttl_days.is_none());
    }

    #[test]
    fn smoke_p1_toggle_and_invite_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
        let bob = DesktopPaths::for_data_root(tmp.path().join("bob"));
        alice.ensure_dirs().unwrap();
        bob.ensure_dirs().unwrap();

        let mut settings = load_or_create_settings(&alice).unwrap();
        apply_network_profile(&mut settings, NetworkProfile::P1, "127.0.0.1:49211", None).unwrap();
        persist_settings(&alice, &settings).unwrap();
        assert_eq!(settings.network_profile, NetworkProfile::P1);
        assert_eq!(settings.peer_listen.as_deref(), Some("127.0.0.1:49211"));

        let (_inv, w, h, rgba) = preview_invite_qr(&alice, &mut settings).unwrap();
        assert!(w >= 256 && h >= 256);
        assert_eq!(rgba.len(), w * h * 4);

        let json = tmp.path().join("alice.json");
        let png = tmp.path().join("alice.png");
        let invite = export_json(&alice, &json, None).unwrap();
        assert_eq!(invite.payload_schema, PEER_INVITE_SCHEMA_ID);
        export_qr(&alice, &png, None).unwrap();

        let applied = import_json(&bob, &json).unwrap();
        assert!(applied.trusted);
        assert!(applied.book_updated);

        let bob2 = DesktopPaths::for_data_root(tmp.path().join("bob2"));
        let applied_qr = import_qr(&bob2, &png).unwrap();
        assert!(applied_qr.trusted);
        assert!(TrustStore::load(&bob2.data_root)
            .unwrap()
            .entries
            .iter()
            .any(|e| e.identity_id == invite.identity_ref));
        assert!(AddressBook::load(&bob2.data_root)
            .unwrap()
            .peers
            .iter()
            .any(|p| p.identity_id == invite.identity_ref));
    }

    #[test]
    fn invite_qr_from_luma_roundtrip_smoke() {
        let tmp = tempfile::tempdir().unwrap();
        let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
        let bob = DesktopPaths::for_data_root(tmp.path().join("bob"));
        alice.ensure_dirs().unwrap();
        bob.ensure_dirs().unwrap();

        let mut settings = load_or_create_settings(&alice).unwrap();
        apply_network_profile(&mut settings, NetworkProfile::P1, "127.0.0.1:49223", None).unwrap();
        persist_settings(&alice, &settings).unwrap();

        let png = tmp.path().join("alice.png");
        let invite = export_qr(&alice, &png, None).unwrap();
        let luma = aira_desktop_runtime::encode_invite_luma(&invite).unwrap();

        let applied = import_qr_luma(&bob, luma).unwrap();
        assert!(applied.trusted);
        assert!(applied.book_updated);
        assert_eq!(applied.identity_ref, invite.identity_ref);
    }

    #[test]
    fn p0_clears_peer_listen() {
        let mut s = DesktopSettings {
            payload_schema: aira_desktop_runtime::SETTINGS_SCHEMA_ID.to_string(),
            network_profile: NetworkProfile::P1,
            open_ui_on_start: true,
            autostart_on_login: false,
            http_listen: "127.0.0.1:8787".into(),
            instance_id: "aira:instance:test".into(),
            http_auth_mode: aira_desktop_runtime::HttpAuthMode::BearerToken,
            http_token_ref: None,
            peer_listen: Some(DEFAULT_PEER_LISTEN.into()),
            relay_ttl_days: None,
        };
        apply_network_profile(&mut s, NetworkProfile::P0, DEFAULT_PEER_LISTEN, None).unwrap();
        assert_eq!(s.network_profile, NetworkProfile::P0);
        assert!(s.peer_listen.is_none());
    }

    #[test]
    fn p5_federation_join_roundtrip() {
        use aira_object::AiraRef;
        use aira_protocol::{
            descriptor_canonical_bytes, FederationDescriptor, FEDERATION_DESCRIPTOR_DOMAIN,
        };
        use ed25519_dalek::SigningKey;

        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let sk = SigningKey::from_bytes(&[77; 32]);
        let pk = hex::encode(sk.verifying_key().to_bytes());
        let id = "aira:identity:fed-gui-104";
        let fed = "aira:federation:gui-104";
        let mut desc = FederationDescriptor {
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
        desc.signature = aira_object::sign_with_key(
            AiraRef::parse(id).unwrap(),
            &sk,
            &descriptor_canonical_bytes(&desc),
        );
        let desc_path = tmp.path().join("fed.json");
        std::fs::write(&desc_path, serde_json::to_string_pretty(&desc).unwrap()).unwrap();

        let out = join_federation_descriptor(&paths, &desc_path).unwrap();
        assert!(!out.already_member);
        let m = federation_membership(&paths).unwrap().unwrap();
        assert_eq!(m.federation_id, fed);
        assert_eq!(m.identity_ref, id);
    }

    #[test]
    fn p6_discovery_stun_fail_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let err = discovery_stun_query(&paths, "").unwrap_err().to_string();
        assert!(err.contains("STUN server required"), "{err}");
    }

    #[test]
    fn submit_problem_empty_fails_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = DesktopPaths::for_data_root(tmp.path());
        let settings = load_or_create_settings(&paths).unwrap();
        let err = submit_problem(&paths, &settings, "  \n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("non-empty"), "{err}");
    }
}
