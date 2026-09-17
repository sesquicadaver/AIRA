//! Safe weights materialization (`#338` / RFC-0222; re-audit R6).
//!
//! - Reject symlinks (fail-closed) before any write effect.
//! - Open source/dest with `O_NOFOLLOW` on Unix.
//! - Stream copy with a bounded buffer (no full-file 2×W buffers).
//! - Exclusive unique `.partial.<id>` temp; writer removes only its own temp.
//! - Post-copy re-hash of the temp path; mismatch deletes that temp; atomic rename.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use aira_object::ContentHash;
use sha2::{Digest, Sha256};

use crate::error::AcquisitionError;

/// Bounded I/O buffer for weights copy/hash (same order as `ContentHash::sha256_path`).
pub(crate) const WEIGHTS_IO_BUF: usize = 64 * 1024;

static PARTIAL_SEQ: AtomicU64 = AtomicU64::new(1);

fn io_err(path: &Path, e: std::io::Error) -> AcquisitionError {
    AcquisitionError::Io(format!("{}: {e}", path.display()))
}

fn reject_if_symlink(path: &Path) -> Result<(), AcquisitionError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(AcquisitionError::SymlinkRejected(
            path.display().to_string(),
        )),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io_err(path, e)),
    }
}

fn map_open_err(path: &Path, e: std::io::Error) -> AcquisitionError {
    #[cfg(unix)]
    {
        if e.raw_os_error() == Some(libc::ELOOP) {
            return AcquisitionError::SymlinkRejected(path.display().to_string());
        }
    }
    io_err(path, e)
}

fn open_nofollow_read(path: &Path) -> Result<File, AcquisitionError> {
    reject_if_symlink(path)?;
    let mut opts = OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW);
    }
    opts.open(path).map_err(|e| map_open_err(path, e))
}

/// Exclusive create — never truncate an existing path (re-audit R6).
fn open_nofollow_create_new(path: &Path) -> Result<File, AcquisitionError> {
    reject_if_symlink(path)?;
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW);
    }
    opts.open(path).map_err(|e| map_open_err(path, e))
}

fn hash_from_sha256(digest: impl AsRef<[u8]>) -> ContentHash {
    ContentHash::parse(format!("sha256:{}", hex::encode(digest.as_ref())))
        .expect("sha256 hex is valid ContentHash")
}

fn stream_hash_file(mut file: File) -> Result<ContentHash, AcquisitionError> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; WEIGHTS_IO_BUF];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| AcquisitionError::Io(e.to_string()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hash_from_sha256(hasher.finalize()))
}

/// Stream-hash a path without following symlinks and without buffering the whole file.
pub(crate) fn content_hash_nofollow(path: &Path) -> Result<ContentHash, AcquisitionError> {
    let file = open_nofollow_read(path)?;
    stream_hash_file(file)
}

/// Unique exclusive sibling temp for one writer (never shared `.partial`).
fn open_exclusive_partial(dest: &Path) -> Result<(PathBuf, File), AcquisitionError> {
    let parent = dest.parent().unwrap_or_else(|| Path::new("."));
    let stem = dest
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("weights.bin");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    for _ in 0u32..64 {
        let seq = PARTIAL_SEQ.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!("{stem}.partial.{nanos}.{seq}"));
        match open_nofollow_create_new(&candidate) {
            Ok(f) => return Ok((candidate, f)),
            Err(e) => {
                // Collision with another exclusive name — retry.
                if candidate.exists() {
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(AcquisitionError::Io(format!(
        "exclusive partial temp exhausted for {}",
        dest.display()
    )))
}

/// Copy `src` → `dest` with no-follow opens, bounded buffer, and post-copy dest hash.
///
/// When `expected` is `Some`, both the streamed copy hash and the post-copy dest hash
/// must equal it (fail-closed; writer temp removed on mismatch).
pub(crate) fn materialize_weights_nofollow(
    src: &Path,
    dest: &Path,
    expected: Option<&ContentHash>,
) -> Result<(ContentHash, u64), AcquisitionError> {
    // Re-audit R6: exclusive unique temp — never shared `.partial`, never delete
    // another writer's file.
    let (tmp, mut dest_f) = open_exclusive_partial(dest)?;
    let mut src_f = match open_nofollow_read(src) {
        Ok(f) => f,
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
    };

    let mut hasher = Sha256::new();
    let mut buf = [0u8; WEIGHTS_IO_BUF];
    let mut bytes: u64 = 0;
    let copy_result = (|| -> Result<(), AcquisitionError> {
        loop {
            let n = src_f
                .read(&mut buf)
                .map_err(|e| AcquisitionError::Io(e.to_string()))?;
            if n == 0 {
                break;
            }
            dest_f
                .write_all(&buf[..n])
                .map_err(|e| AcquisitionError::Io(e.to_string()))?;
            hasher.update(&buf[..n]);
            bytes += n as u64;
        }
        dest_f
            .sync_all()
            .map_err(|e| AcquisitionError::Io(e.to_string()))?;
        Ok(())
    })();

    drop(dest_f);

    if let Err(e) = copy_result {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }

    let streamed = hash_from_sha256(hasher.finalize());
    if let Some(exp) = expected {
        if streamed != *exp {
            let _ = fs::remove_file(&tmp);
            return Err(AcquisitionError::MaterializeHashMismatch {
                expected: exp.as_str().to_string(),
                observed: streamed.as_str().to_string(),
            });
        }
    }

    let post = match content_hash_nofollow(&tmp) {
        Ok(h) => h,
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
    };
    if let Some(exp) = expected {
        if post != *exp {
            let _ = fs::remove_file(&tmp);
            return Err(AcquisitionError::MaterializeHashMismatch {
                expected: exp.as_str().to_string(),
                observed: post.as_str().to_string(),
            });
        }
    }
    if streamed != post {
        let _ = fs::remove_file(&tmp);
        return Err(AcquisitionError::MaterializeHashMismatch {
            expected: streamed.as_str().to_string(),
            observed: post.as_str().to_string(),
        });
    }

    if dest.exists() {
        if let Err(e) = reject_if_symlink(dest) {
            let _ = fs::remove_file(&tmp);
            return Err(e);
        }
    }
    fs::rename(&tmp, dest).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        AcquisitionError::Io(format!("atomic publish {}: {e}", dest.display()))
    })?;
    Ok((post, bytes))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn content_hash_nofollow_rejects_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real.bin");
        let link = dir.path().join("link.bin");
        fs::write(&real, b"abc").unwrap();
        symlink(&real, &link).unwrap();
        let err = content_hash_nofollow(&link).unwrap_err();
        assert!(matches!(err, AcquisitionError::SymlinkRejected(_)));
    }

    #[test]
    fn materialize_rejects_dest_symlink_leaves_target() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src.bin");
        let outside = dir.path().join("outside.bin");
        let dest = dir.path().join("dest.bin");
        fs::write(&src, b"payload-bytes").unwrap();
        fs::write(&outside, b"do-not-clobber").unwrap();
        symlink(&outside, &dest).unwrap();
        let err = materialize_weights_nofollow(&src, &dest, None).unwrap_err();
        assert!(matches!(err, AcquisitionError::SymlinkRejected(_)));
        assert_eq!(fs::read(&outside).unwrap(), b"do-not-clobber");
    }

    #[test]
    fn materialize_streams_and_post_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src.bin");
        let dest = dir.path().join("dest.bin");
        let payload = b"stream-copy-payload";
        fs::write(&src, payload).unwrap();
        let expected = ContentHash::sha256_bytes(payload);
        let (got, bytes) = materialize_weights_nofollow(&src, &dest, Some(&expected)).unwrap();
        assert_eq!(got, expected);
        assert_eq!(bytes, payload.len() as u64);
        assert_eq!(fs::read(&dest).unwrap(), payload);
    }

    #[test]
    fn materialize_expected_mismatch_removes_dest() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src.bin");
        let dest = dir.path().join("dest.bin");
        fs::write(&src, b"actual").unwrap();
        let wrong = ContentHash::sha256_bytes(b"other");
        let err = materialize_weights_nofollow(&src, &dest, Some(&wrong)).unwrap_err();
        assert!(matches!(
            err,
            AcquisitionError::MaterializeHashMismatch { .. }
        ));
        assert!(!dest.exists());
        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".partial."))
            .collect();
        assert!(
            leftovers.is_empty(),
            "writer must remove its own temp, got {leftovers:?}"
        );
    }

    /// Re-audit R6: two writers use distinct exclusive temps; dest ends as one complete payload.
    #[test]
    fn concurrent_materialize_unique_temps_publish_safely() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("dest.bin");
        let src_a = dir.path().join("src-a.bin");
        let src_b = dir.path().join("src-b.bin");
        let payload_a = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let payload_b = b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        fs::write(&src_a, payload_a).unwrap();
        fs::write(&src_b, payload_b).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let dest_a = dest.clone();
        let dest_b = dest.clone();
        let ba = barrier.clone();
        let bb = barrier.clone();
        let ha = thread::spawn(move || {
            ba.wait();
            materialize_weights_nofollow(&src_a, &dest_a, None)
        });
        let hb = thread::spawn(move || {
            bb.wait();
            materialize_weights_nofollow(&src_b, &dest_b, None)
        });
        let ra = ha.join().unwrap();
        let rb = hb.join().unwrap();
        assert!(ra.is_ok() || rb.is_ok(), "at least one writer must publish");
        let got = fs::read(&dest).unwrap();
        assert!(
            got == payload_a || got == payload_b,
            "dest must be atomic complete payload, got {} bytes",
            got.len()
        );
        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".partial."))
            .collect();
        assert!(
            leftovers.is_empty(),
            "no orphan exclusive temps, got {leftovers:?}"
        );
    }
}
