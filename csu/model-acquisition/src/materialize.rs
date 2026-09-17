//! Safe weights materialization (`#338` / RFC-0222).
//!
//! - Reject symlinks (fail-closed) before any write effect.
//! - Open source/dest with `O_NOFOLLOW` on Unix.
//! - Stream copy with a bounded buffer (no full-file 2×W buffers).
//! - Post-copy re-hash of the dest path; mismatch deletes dest.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use aira_object::ContentHash;
use sha2::{Digest, Sha256};

use crate::error::AcquisitionError;

/// Bounded I/O buffer for weights copy/hash (same order as `ContentHash::sha256_path`).
pub(crate) const WEIGHTS_IO_BUF: usize = 64 * 1024;

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

fn open_nofollow_write(path: &Path) -> Result<File, AcquisitionError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(AcquisitionError::SymlinkRejected(
            path.display().to_string(),
        )),
        Ok(_) => {
            let mut opts = OpenOptions::new();
            opts.write(true).truncate(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.custom_flags(libc::O_NOFOLLOW);
            }
            opts.open(path).map_err(|e| map_open_err(path, e))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut opts = OpenOptions::new();
            opts.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.custom_flags(libc::O_NOFOLLOW);
            }
            opts.open(path).map_err(|e| map_open_err(path, e))
        }
        Err(e) => Err(io_err(path, e)),
    }
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

/// Copy `src` → `dest` with no-follow opens, bounded buffer, and post-copy dest hash.
///
/// When `expected` is `Some`, both the streamed copy hash and the post-copy dest hash
/// must equal it (fail-closed; dest removed on mismatch).
pub(crate) fn materialize_weights_nofollow(
    src: &Path,
    dest: &Path,
    expected: Option<&ContentHash>,
) -> Result<(ContentHash, u64), AcquisitionError> {
    // Pack D / audit #4: write to a sibling temp then atomic rename so an
    // in-flight reader never sees a truncated dest.
    let tmp = dest.with_extension("partial");
    if tmp.exists() {
        let _ = fs::remove_file(&tmp);
    }
    let mut src_f = open_nofollow_read(src)?;
    let mut dest_f = open_nofollow_write(&tmp)?;

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

    let post = content_hash_nofollow(&tmp)?;
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
        reject_if_symlink(dest)?;
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
    }
}
