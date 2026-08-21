//! PeerInvite QR PNG encode/decode (QUEUE #84 / Analyze-119).
//!
//! Payload is compact JSON of the same `PeerInvite` schema as file IO.
//! Camera / live scan is Out — decode only from an image file on disk.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use image::Luma;
use qrcode::QrCode;

use crate::bootstrap::ensure_bootstrap;
use crate::invite::{
    build_local_invite, import_invite, validate_peer_invite, ImportInviteOutcome, PeerInvite,
};
use crate::paths::DesktopPaths;
use crate::settings::load_or_create_settings;

/// Compact JSON bytes encoded into the QR (no pretty-print).
pub fn invite_qr_payload(invite: &PeerInvite) -> Result<String> {
    validate_peer_invite(invite)?;
    serde_json::to_string(invite).context("serialize PeerInvite for QR")
}

/// Render invite QR as grayscale pixels (for GUI preview / PNG encode).
pub fn encode_invite_luma(invite: &PeerInvite) -> Result<image::GrayImage> {
    let payload = invite_qr_payload(invite)?;
    let code = QrCode::new(payload.as_bytes())
        .with_context(|| format!("QR encode failed (payload {} bytes)", payload.len()))?;
    Ok(code.render::<Luma<u8>>().min_dimensions(256, 256).build())
}

/// RGBA preview of invite QR (`width`, `height`, interleaved pixels).
pub fn encode_invite_rgba(invite: &PeerInvite) -> Result<(usize, usize, Vec<u8>)> {
    let img = encode_invite_luma(invite)?;
    let (w, h) = img.dimensions();
    let mut rgba = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for p in img.pixels() {
        let v = p.0[0];
        rgba.extend_from_slice(&[v, v, v, 255]);
    }
    Ok((w as usize, h as usize, rgba))
}

/// Encode a validated PeerInvite as a PNG QR at `out_path`.
pub fn encode_invite_png(invite: &PeerInvite, out_path: &Path) -> Result<()> {
    let img = encode_invite_luma(invite)?;
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    img.save(out_path)
        .with_context(|| format!("write PNG {}", out_path.display()))?;
    Ok(())
}

/// Decode the first QR found in a PNG/image file into a validated PeerInvite.
pub fn decode_invite_png(path: &Path) -> Result<PeerInvite> {
    let img = image::open(path)
        .with_context(|| format!("open image {}", path.display()))?
        .to_luma8();
    let mut prep = rqrr::PreparedImage::prepare(img);
    let grids = prep.detect_grids();
    if grids.is_empty() {
        bail!("no QR code found in {}", path.display());
    }
    let mut last_err = None;
    for grid in &grids {
        match grid.decode() {
            Ok((_meta, content)) => match serde_json::from_str::<PeerInvite>(&content) {
                Ok(invite) => {
                    validate_peer_invite(&invite)?;
                    return Ok(invite);
                }
                Err(e) => {
                    last_err = Some(anyhow::anyhow!("QR payload is not PeerInvite JSON: {e}"));
                }
            },
            Err(e) => {
                last_err = Some(anyhow::anyhow!("QR decode: {e}"));
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("QR decode failed")))
}

/// Export local invite as a PNG QR file.
pub fn export_invite_qr_png(
    paths: &DesktopPaths,
    out_path: &Path,
    addr_override: Option<String>,
) -> Result<PeerInvite> {
    paths.ensure_dirs()?;
    let mut settings = load_or_create_settings(paths)?;
    ensure_bootstrap(paths, &mut settings)?;
    let invite = build_local_invite(paths, &settings, addr_override)?;
    encode_invite_png(&invite, out_path)?;
    Ok(invite)
}

/// Import PeerInvite from a QR PNG/image file → trust + optional address book.
pub fn import_invite_qr_file(paths: &DesktopPaths, path: &Path) -> Result<ImportInviteOutcome> {
    let invite = decode_invite_png(path)?;
    import_invite(paths, &invite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite::PEER_INVITE_SCHEMA_ID;

    #[test]
    fn reject_non_invite_qr_payload() {
        let tmp = tempfile::tempdir().unwrap();
        let png = tmp.path().join("hello.png");
        let code = QrCode::new(b"hello-not-invite").unwrap();
        code.render::<Luma<u8>>()
            .min_dimensions(128, 128)
            .build()
            .save(&png)
            .unwrap();
        let err = decode_invite_png(&png).unwrap_err().to_string();
        assert!(err.contains("PeerInvite") || err.contains("JSON"), "{err}");
    }

    #[test]
    fn reject_blank_png_no_qr() {
        let tmp = tempfile::tempdir().unwrap();
        let png = tmp.path().join("blank.png");
        let img = image::GrayImage::from_pixel(64, 64, Luma([255]));
        img.save(&png).unwrap();
        let err = decode_invite_png(&png).unwrap_err().to_string();
        assert!(err.contains("no QR"), "{err}");
    }

    #[test]
    fn payload_is_compact_json() {
        let invite = PeerInvite {
            payload_schema: PEER_INVITE_SCHEMA_ID.to_string(),
            identity_ref: "aira:identity:desktop".into(),
            public_key_hex: "d4295b4daeeb41c8dcc7ab0823210104b257a68f38f79d26bdd66875265e0444"
                .into(),
            addr: Some("127.0.0.1:9797".into()),
            display_name: None,
            created_at: None,
        };
        let payload = invite_qr_payload(&invite).unwrap();
        assert!(!payload.contains('\n'));
        assert!(payload.starts_with('{'));
    }
}
