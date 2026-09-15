//! Model storage paths + disk usage for Settings (`#355` / RFC-0238).
//!
//! Shows the local `models/` tree under the Desktop data root, used bytes, and
//! volume free space when observable. Fail-closed: unavailable free space →
//! `None` (UI shows unknown). Not marketplace, delete, or path editing.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Frozen storage view for Settings → Models (`#355`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelStorageSnapshot {
    /// `<data_root>/models` (inventory scoped root).
    pub models_root: PathBuf,
    pub quarantine_dir: PathBuf,
    pub verified_dir: PathBuf,
    pub cache_dir: PathBuf,
    /// Sum of regular-file sizes under `models/` (symlinks not followed).
    pub used_bytes: u64,
    /// Free bytes on the volume containing `models_root`, when known.
    pub available_bytes: Option<u64>,
}

impl Default for ModelStorageSnapshot {
    fn default() -> Self {
        Self {
            models_root: PathBuf::from("models"),
            quarantine_dir: PathBuf::from("models/quarantine"),
            verified_dir: PathBuf::from("models/verified"),
            cache_dir: PathBuf::from("models/cache"),
            used_bytes: 0,
            available_bytes: None,
        }
    }
}

/// Resolve model storage layout under an AIRA data root.
pub fn load_model_storage(root: impl AsRef<Path>) -> ModelStorageSnapshot {
    let models_root = root.as_ref().join("models");
    let quarantine_dir = models_root.join("quarantine");
    let verified_dir = models_root.join("verified");
    let cache_dir = models_root.join("cache");
    let used_bytes = dir_used_bytes(&models_root);
    // Probe the models dir when present; otherwise the parent data root volume.
    let probe = if models_root.exists() {
        models_root.as_path()
    } else {
        root.as_ref()
    };
    let available_bytes = volume_available_bytes(probe);
    ModelStorageSnapshot {
        models_root,
        quarantine_dir,
        verified_dir,
        cache_dir,
        used_bytes,
        available_bytes,
    }
}

/// Human-readable size (binary units). Empty / zero → `"0 B"`.
pub fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;
    if bytes >= TIB {
        format!("{:.1} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Sum regular-file sizes under `path`. Missing path → `0`. Does not follow
/// directory symlinks (escape fail-closed).
pub fn dir_used_bytes(path: &Path) -> u64 {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if meta.file_type().is_symlink() {
        // Top-level symlink: do not follow out of the tree.
        return 0;
    }
    if meta.is_file() {
        return meta.len();
    }
    if !meta.is_dir() {
        return 0;
    }
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let meta = match fs::symlink_metadata(&p) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let ft = meta.file_type();
            if ft.is_symlink() {
                continue;
            }
            if ft.is_file() {
                total = total.saturating_add(meta.len());
            } else if ft.is_dir() {
                stack.push(p);
            }
        }
    }
    total
}

/// Free bytes on the filesystem volume for `path`, or `None` if unknown.
pub fn volume_available_bytes(path: &Path) -> Option<u64> {
    volume_available_bytes_impl(path)
}

#[cfg(unix)]
fn volume_available_bytes_impl(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: `c` is a valid NUL-terminated path; `statvfs` fills `buf`.
    unsafe {
        let mut buf: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c.as_ptr(), &mut buf) != 0 {
            return None;
        }
        // `f_frsize` / `f_bavail` are already `u64` on Linux glibc; width varies
        // across libc targets — multiply in place without redundant casts.
        Some(buf.f_frsize.saturating_mul(buf.f_bavail))
    }
}

#[cfg(not(unix))]
fn volume_available_bytes_impl(_path: &Path) -> Option<u64> {
    // Fail-closed: no invented free-space figure on unsupported hosts.
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn missing_models_dir_is_zero_used() {
        let dir = tempdir().unwrap();
        let snap = load_model_storage(dir.path());
        assert_eq!(snap.used_bytes, 0);
        assert!(snap.models_root.ends_with("models"));
        assert!(snap.quarantine_dir.ends_with("quarantine"));
        assert!(snap.verified_dir.ends_with("verified"));
        assert!(snap.cache_dir.ends_with("cache"));
    }

    #[test]
    fn used_bytes_counts_regular_files() {
        let dir = tempdir().unwrap();
        let models = dir.path().join("models");
        fs::create_dir_all(models.join("cache/slot-a")).unwrap();
        let mut f = fs::File::create(models.join("cache/slot-a/weights.bin")).unwrap();
        f.write_all(&[0u8; 4096]).unwrap();
        drop(f);
        let snap = load_model_storage(dir.path());
        assert!(
            snap.used_bytes >= 4096,
            "expected ≥4096, got {}",
            snap.used_bytes
        );
    }

    #[test]
    fn symlink_dir_not_followed_for_used() {
        let dir = tempdir().unwrap();
        let models = dir.path().join("models");
        fs::create_dir_all(&models).unwrap();
        let outside = dir.path().join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("big.bin"), vec![1u8; 8192]).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside, models.join("escape")).unwrap();
            let used = dir_used_bytes(&models);
            assert_eq!(used, 0, "symlink escape must not count outside bytes");
        }
    }

    #[test]
    fn format_bytes_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert!(format_bytes(2048).contains("KiB"));
        assert!(format_bytes(2 * 1024 * 1024).contains("MiB"));
    }

    #[test]
    #[cfg(unix)]
    fn available_bytes_some_on_temp() {
        let dir = tempdir().unwrap();
        let avail = volume_available_bytes(dir.path());
        assert!(avail.is_some(), "temp volume should report free space");
        assert!(avail.unwrap() > 0);
    }
}
