//! Noise XX transport after Ed25519 hello (Analyze-35).
//! Analyze-49: coordinated rotate of `identity/local.x25519` with Ed25519 identity rotate.

use std::path::{Path, PathBuf};

use snow::{Builder, TransportState};
use tokio::net::TcpStream;
use x25519_dalek::{PublicKey, StaticSecret};

use crate::error::PeerError;
use crate::frame::{read_frame, write_frame};

/// Noise pattern for peer links.
pub const NOISE_PATTERN: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";

const NODE_X25519_FILE: &str = "local.x25519";
/// Opt-in previous Noise static backup (Analyze-49).
pub const NODE_X25519_BACKUP_FILE: &str = "local.x25519.prev";

/// One Noise static backup slot (latest or archived stamp).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoiseStaticBackupInfo {
    pub stamp: String,
    pub secret_path: PathBuf,
    pub is_latest: bool,
}

/// List Noise static backups (latest + `local.x25519.prev.<stamp>`).
pub fn list_noise_static_backups(
    root: impl AsRef<Path>,
) -> Result<Vec<NoiseStaticBackupInfo>, PeerError> {
    let identity_dir = root.as_ref().join("identity");
    if !identity_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let latest = identity_dir.join(NODE_X25519_BACKUP_FILE);
    if latest.is_file() {
        out.push(NoiseStaticBackupInfo {
            stamp: "latest".into(),
            secret_path: latest,
            is_latest: true,
        });
    }
    let prefix = format!("{NODE_X25519_BACKUP_FILE}.");
    let rd = std::fs::read_dir(&identity_dir).map_err(|e| PeerError::Io(e.to_string()))?;
    for ent in rd {
        let ent = ent.map_err(|e| PeerError::Io(e.to_string()))?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(&prefix) {
            continue;
        }
        let stamp = name[prefix.len()..].to_string();
        if stamp.is_empty() || stamp.contains('.') {
            continue;
        }
        let path = ent.path();
        if !path.is_file() {
            continue;
        }
        out.push(NoiseStaticBackupInfo {
            stamp,
            secret_path: path,
            is_latest: false,
        });
    }
    out.sort_by(|a, b| match (a.is_latest, b.is_latest) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.stamp.cmp(&a.stamp),
    });
    Ok(out)
}

/// Result of [`prune_noise_static_backups`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoiseStaticPruneReport {
    pub deleted: Vec<PathBuf>,
    pub skipped: Vec<(PathBuf, String)>,
    pub dry_run: bool,
}

fn x25519_stamp_unix(stamp: &str) -> Option<i64> {
    // Analyze-49 stamps are `<unix-secs>Z` (optional `-N` collision suffix).
    let base = stamp.split('-').next().unwrap_or(stamp);
    let digits = base.strip_suffix('Z')?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn noise_retain(
    rank: u64,
    age_unix: Option<i64>,
    keep: Option<u64>,
    older_than_days: Option<u64>,
    now: i64,
) -> Result<bool, String> {
    let keep_ok = match keep {
        None => true,
        Some(n) => rank < n,
    };
    let age_ok = match older_than_days {
        None => true,
        Some(days) => {
            let age = age_unix.ok_or_else(|| "unparseable age".to_string())?;
            let limit = i64::try_from(days)
                .unwrap_or(i64::MAX)
                .saturating_mul(86_400);
            now.saturating_sub(age) <= limit
        }
    };
    Ok(keep_ok && age_ok)
}

/// Prune archived `local.x25519.prev.<stamp>` slots (Analyze-61). Never deletes latest.
pub fn prune_noise_static_backups(
    root: impl AsRef<Path>,
    keep: Option<u64>,
    older_than_days: Option<u64>,
    dry_run: bool,
) -> Result<NoiseStaticPruneReport, PeerError> {
    if keep.is_none() && older_than_days.is_none() {
        return Err(PeerError::Io(
            "prune requires --keep and/or --older-than-days".into(),
        ));
    }
    let mut report = NoiseStaticPruneReport {
        dry_run,
        ..Default::default()
    };
    let list = list_noise_static_backups(&root)?;
    let mut archived: Vec<_> = list.into_iter().filter(|b| !b.is_latest).collect();
    archived.sort_by(|a, b| b.stamp.cmp(&a.stamp));
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    for (rank, info) in archived.into_iter().enumerate() {
        let rank = rank as u64;
        let age = x25519_stamp_unix(&info.stamp);
        match noise_retain(rank, age, keep, older_than_days, now) {
            Ok(true) => {}
            Ok(false) => {
                if dry_run {
                    report.deleted.push(info.secret_path.clone());
                } else {
                    std::fs::remove_file(&info.secret_path).map_err(|e| {
                        PeerError::Io(format!("prune {}: {e}", info.secret_path.display()))
                    })?;
                    report.deleted.push(info.secret_path.clone());
                }
            }
            Err(reason) => {
                report.skipped.push((info.secret_path.clone(), reason));
            }
        }
    }
    Ok(report)
}

/// Result of [`rotate_noise_static`].
#[derive(Debug, Clone)]
pub struct NoiseStaticRotate {
    /// Previous public key hex when a prior secret existed.
    pub old_public_hex: Option<String>,
    /// New public key hex after rotate.
    pub new_public_hex: String,
    /// Path to `local.x25519.prev` when `backup` succeeded with a prior secret.
    pub backup_path: Option<PathBuf>,
}

/// Load or create a persistent X25519 static secret under `identity/local.x25519`.
pub fn load_or_create_noise_static(root: impl AsRef<Path>) -> Result<[u8; 32], PeerError> {
    let path = root.as_ref().join("identity").join(NODE_X25519_FILE);
    if path.exists() {
        return read_x25519_secret(&path);
    }
    let secret = StaticSecret::random_from_rng(rand::thread_rng());
    let bytes = secret.to_bytes();
    write_x25519_secret(&path, &bytes)?;
    Ok(bytes)
}

/// Rotate the Noise static secret (Analyze-49).
///
/// Always writes a fresh `identity/local.x25519`. When `backup` is true and a prior
/// secret exists, stages it to `local.x25519.prev` (archiving any existing `.prev`
/// to `local.x25519.prev.<UTC stamp>` first). Mode `0600` on Unix.
pub fn rotate_noise_static(
    root: impl AsRef<Path>,
    backup: bool,
) -> Result<NoiseStaticRotate, PeerError> {
    let identity_dir = root.as_ref().join("identity");
    let path = identity_dir.join(NODE_X25519_FILE);
    let backup_path = identity_dir.join(NODE_X25519_BACKUP_FILE);
    std::fs::create_dir_all(&identity_dir).map_err(|e| PeerError::Io(e.to_string()))?;

    let old_secret = if path.is_file() {
        Some(read_x25519_secret(&path)?)
    } else {
        None
    };
    let old_public_hex = old_secret.as_ref().map(|s| hex::encode(x25519_public(s)));

    let mut wrote_backup: Option<PathBuf> = None;
    if backup {
        if let Some(ref old) = old_secret {
            archive_x25519_prev_if_present(&identity_dir)?;
            let encoded = hex::encode(old);
            write_secret_file(&backup_path, encoded.as_bytes())?;
            wrote_backup = Some(backup_path);
        }
    }

    let new = StaticSecret::random_from_rng(rand::thread_rng());
    let new_bytes = new.to_bytes();
    write_x25519_secret(&path, &new_bytes)?;
    Ok(NoiseStaticRotate {
        old_public_hex,
        new_public_hex: hex::encode(x25519_public(&new_bytes)),
        backup_path: wrote_backup,
    })
}

fn read_x25519_secret(path: &Path) -> Result<[u8; 32], PeerError> {
    let raw = std::fs::read_to_string(path).map_err(|e| PeerError::Io(e.to_string()))?;
    let raw = raw.trim();
    let bytes = hex::decode(raw).map_err(|e| PeerError::Crypto(format!("x25519 hex: {e}")))?;
    if bytes.len() != 32 {
        return Err(PeerError::Crypto(format!(
            "x25519 secret must be 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn write_x25519_secret(path: &Path, bytes: &[u8; 32]) -> Result<(), PeerError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| PeerError::Io(e.to_string()))?;
    }
    let encoded = hex::encode(bytes);
    write_secret_file(path, encoded.as_bytes())
}

fn write_secret_file(path: &Path, contents: &[u8]) -> Result<(), PeerError> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        // Prefer create_new for first write; fall back to truncate for rotate overwrite.
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).mode(0o600);
        if path.exists() {
            opts.truncate(true);
        } else {
            opts.create_new(true);
        }
        let mut file = opts.open(path).map_err(|e| PeerError::Io(e.to_string()))?;
        file.write_all(contents)
            .map_err(|e| PeerError::Io(e.to_string()))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, contents).map_err(|e| PeerError::Io(e.to_string()))?;
    }
    Ok(())
}

fn archive_x25519_prev_if_present(identity_dir: &Path) -> Result<(), PeerError> {
    let latest = identity_dir.join(NODE_X25519_BACKUP_FILE);
    if !latest.is_file() {
        return Ok(());
    }
    let stamp = utc_compact_stamp();
    let mut archived = identity_dir.join(format!("{NODE_X25519_BACKUP_FILE}.{stamp}"));
    let mut n = 2u32;
    while archived.exists() {
        archived = identity_dir.join(format!("{NODE_X25519_BACKUP_FILE}.{stamp}-{n}"));
        n += 1;
    }
    std::fs::rename(&latest, &archived).map_err(|e| {
        PeerError::Io(format!(
            "archive x25519.prev failed ({} → {}): {e}",
            latest.display(),
            archived.display()
        ))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&archived, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

fn utc_compact_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Compact UTC-ish stamp without chrono dep in this crate path.
    format!("{secs}Z")
}

/// Public X25519 bytes for a static secret.
pub fn x25519_public(secret: &[u8; 32]) -> [u8; 32] {
    let sk = StaticSecret::from(*secret);
    PublicKey::from(&sk).to_bytes()
}

fn builder(static_priv: &[u8; 32]) -> Result<Builder<'_>, PeerError> {
    let params = NOISE_PATTERN
        .parse()
        .map_err(|e| PeerError::Crypto(format!("noise params: {e}")))?;
    Builder::new(params)
        .local_private_key(static_priv)
        .map_err(|e| PeerError::Crypto(format!("noise local key: {e}")))
}

async fn write_noise_msg(stream: &mut TcpStream, payload: &[u8]) -> Result<(), PeerError> {
    write_frame(stream, payload).await
}

async fn read_noise_msg(stream: &mut TcpStream) -> Result<Vec<u8>, PeerError> {
    read_frame(stream).await
}

/// Initiator XX after hello. Returns transport + remote static public key.
pub async fn noise_xx_initiator(
    stream: &mut TcpStream,
    static_priv: &[u8; 32],
) -> Result<(TransportState, [u8; 32]), PeerError> {
    let mut noise = builder(static_priv)?
        .build_initiator()
        .map_err(|e| PeerError::Crypto(format!("noise initiator: {e}")))?;
    let mut buf = vec![0u8; 65535];
    let mut tmp = vec![0u8; 65535];

    // -> e
    let n = noise
        .write_message(&[], &mut buf)
        .map_err(|e| PeerError::Crypto(format!("noise write1: {e}")))?;
    write_noise_msg(stream, &buf[..n]).await?;

    // <- e, ee, s, es
    let msg = read_noise_msg(stream).await?;
    noise
        .read_message(&msg, &mut tmp)
        .map_err(|e| PeerError::Crypto(format!("noise read2: {e}")))?;

    // -> s, se
    let n = noise
        .write_message(&[], &mut buf)
        .map_err(|e| PeerError::Crypto(format!("noise write3: {e}")))?;
    write_noise_msg(stream, &buf[..n]).await?;

    let remote = noise
        .get_remote_static()
        .ok_or_else(|| PeerError::Crypto("noise missing remote static".into()))?
        .to_vec();
    if remote.len() != 32 {
        return Err(PeerError::Crypto("noise remote static len".into()));
    }
    let mut remote_arr = [0u8; 32];
    remote_arr.copy_from_slice(&remote);

    let transport = noise
        .into_transport_mode()
        .map_err(|e| PeerError::Crypto(format!("noise transport: {e}")))?;
    Ok((transport, remote_arr))
}

/// Responder XX after hello. Returns transport + remote static public key.
pub async fn noise_xx_responder(
    stream: &mut TcpStream,
    static_priv: &[u8; 32],
) -> Result<(TransportState, [u8; 32]), PeerError> {
    let mut noise = builder(static_priv)?
        .build_responder()
        .map_err(|e| PeerError::Crypto(format!("noise responder: {e}")))?;
    let mut buf = vec![0u8; 65535];
    let mut tmp = vec![0u8; 65535];

    // <- e
    let msg = read_noise_msg(stream).await?;
    noise
        .read_message(&msg, &mut tmp)
        .map_err(|e| PeerError::Crypto(format!("noise read1: {e}")))?;

    // -> e, ee, s, es
    let n = noise
        .write_message(&[], &mut buf)
        .map_err(|e| PeerError::Crypto(format!("noise write2: {e}")))?;
    write_noise_msg(stream, &buf[..n]).await?;

    // <- s, se
    let msg = read_noise_msg(stream).await?;
    noise
        .read_message(&msg, &mut tmp)
        .map_err(|e| PeerError::Crypto(format!("noise read3: {e}")))?;

    let remote = noise
        .get_remote_static()
        .ok_or_else(|| PeerError::Crypto("noise missing remote static".into()))?
        .to_vec();
    if remote.len() != 32 {
        return Err(PeerError::Crypto("noise remote static len".into()));
    }
    let mut remote_arr = [0u8; 32];
    remote_arr.copy_from_slice(&remote);

    let transport = noise
        .into_transport_mode()
        .map_err(|e| PeerError::Crypto(format!("noise transport: {e}")))?;
    Ok((transport, remote_arr))
}

/// Encrypt plaintext and write as one length-prefixed frame.
pub async fn write_encrypted(
    stream: &mut TcpStream,
    transport: &mut TransportState,
    plaintext: &[u8],
) -> Result<(), PeerError> {
    let mut buf = vec![0u8; plaintext.len() + 64];
    let n = transport
        .write_message(plaintext, &mut buf)
        .map_err(|e| PeerError::Crypto(format!("noise encrypt: {e}")))?;
    write_frame(stream, &buf[..n]).await
}

/// Read one frame and decrypt.
pub async fn read_encrypted(
    stream: &mut TcpStream,
    transport: &mut TransportState,
) -> Result<Vec<u8>, PeerError> {
    let frame = read_frame(stream).await?;
    let mut buf = vec![0u8; frame.len()];
    let n = transport
        .read_message(&frame, &mut buf)
        .map_err(|e| PeerError::Crypto(format!("noise decrypt: {e}")))?;
    buf.truncate(n);
    Ok(buf)
}
