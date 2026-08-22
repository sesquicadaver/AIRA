//! Testable Desktop GUI actions (QUEUE #85) — no egui dependency.

use std::path::Path;

use anyhow::Result;

use aira_desktop_runtime::{
    build_local_invite, encode_invite_rgba, ensure_bootstrap, export_invite_file,
    export_invite_qr_png, import_invite_file, import_invite_qr_file, normalize_settings,
    write_settings, DesktopPaths, DesktopSettings, ImportInviteOutcome, NetworkProfile, PeerInvite,
    DEFAULT_PEER_LISTEN,
};

/// Apply P0, P1, or P2 profile; fill default `peer_listen` on P1/P2.
pub fn apply_network_profile(
    settings: &mut DesktopSettings,
    profile: NetworkProfile,
    peer_listen_edit: &str,
) -> Result<()> {
    if !profile.is_supported() {
        anyhow::bail!("unsupported network_profile {profile:?}");
    }
    settings.network_profile = profile;
    match profile {
        NetworkProfile::P1 | NetworkProfile::P2 => {
            let trimmed = peer_listen_edit.trim();
            settings.peer_listen = Some(if trimmed.is_empty() {
                DEFAULT_PEER_LISTEN.to_string()
            } else {
                trimmed.to_string()
            });
        }
        NetworkProfile::P0 => {
            // Keep last peer_listen value on disk optional; clear for P0 UI clarity.
            settings.peer_listen = None;
        }
        _ => unreachable!("is_supported"),
    }
    normalize_settings(settings)
}

/// Persist settings after profile/listen edits.
pub fn persist_settings(paths: &DesktopPaths, settings: &DesktopSettings) -> Result<()> {
    write_settings(paths, settings)
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

#[cfg(test)]
mod tests {
    use super::*;
    use aira_desktop_runtime::{load_or_create_settings, PEER_INVITE_SCHEMA_ID};
    use aira_object::TrustStore;
    use aira_peer::AddressBook;

    #[test]
    fn smoke_p1_toggle_and_invite_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let alice = DesktopPaths::for_data_root(tmp.path().join("alice"));
        let bob = DesktopPaths::for_data_root(tmp.path().join("bob"));
        alice.ensure_dirs().unwrap();
        bob.ensure_dirs().unwrap();

        let mut settings = load_or_create_settings(&alice).unwrap();
        apply_network_profile(&mut settings, NetworkProfile::P1, "127.0.0.1:19085").unwrap();
        persist_settings(&alice, &settings).unwrap();
        assert_eq!(settings.network_profile, NetworkProfile::P1);
        assert_eq!(settings.peer_listen.as_deref(), Some("127.0.0.1:19085"));

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
        };
        apply_network_profile(&mut s, NetworkProfile::P0, DEFAULT_PEER_LISTEN).unwrap();
        assert_eq!(s.network_profile, NetworkProfile::P0);
        assert!(s.peer_listen.is_none());
    }
}
