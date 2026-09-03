//! Linux Landlock FS restrict for the generate-local child (QUEUE #225).
//!
//! Applied in the child after fork, before exec ([`std::os::unix::process::CommandExt::pre_exec`]).
//! Handled FS accesses are ABI-1; allowlist paths get execute + read only. Writes,
//! creates, and paths outside the allowlist are denied by the kernel.
//!
//! Opt-in only. Missing kernel / failed restrict → fail-closed (not unsandboxed
//! success). netns is a later atom.

use std::io::Error;
#[cfg(not(target_os = "linux"))]
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Fail-closed when Landlock cannot be applied (kernel, path, or restrict).
pub const LANDLOCK_FAILED: &str =
    "generate process landlock restrict failed (fail-closed; not VERIFIED)";

/// Fail-closed when Landlock is requested on a non-Linux host.
pub const LANDLOCK_UNSUPPORTED: &str =
    "generate process landlock unsupported (fail-closed; not VERIFIED)";

/// `AIRA_LLM_LANDLOCK=1|true|yes` enables FS restrict on [`super::ProcessBackend`].
pub const ENV_LLM_LANDLOCK: &str = "AIRA_LLM_LANDLOCK";

/// Interpreter and linker dirs so a shebang script under the jail can still start.
pub const LANDLOCK_RUNTIME_DIRS: &[&str] = &[
    "/bin",
    "/usr/bin",
    "/lib",
    "/lib64",
    "/usr/lib",
    "/usr/lib64",
    "/etc",
    "/dev",
];

/// Parse the Landlock opt-in env value. Unset / anything else → off.
pub fn landlock_enabled_from(value: Option<&str>) -> bool {
    match value {
        Some(v) if v.eq_ignore_ascii_case("1") => true,
        Some(v) if v.eq_ignore_ascii_case("true") => true,
        Some(v) if v.eq_ignore_ascii_case("yes") => true,
        _ => false,
    }
}

/// Allow the program directory plus runtime dirs that exist on this host.
pub fn default_allow_paths(program: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(parent) = program.parent() {
        if !parent.as_os_str().is_empty() {
            out.push(parent.to_path_buf());
        }
    }
    for dir in LANDLOCK_RUNTIME_DIRS {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            out.push(p);
        }
    }
    out
}

/// Restrict the current thread (child) to `allow` directories.
pub fn restrict_fs_self(allow: &[PathBuf]) -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        linux::restrict_fs_self(allow)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = allow;
        Err(Error::new(ErrorKind::Unsupported, LANDLOCK_UNSUPPORTED))
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::LANDLOCK_FAILED;
    use std::ffi::CString;
    use std::io::{Error, ErrorKind};
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::ptr;

    const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1 << 0;
    const LANDLOCK_RULE_PATH_BENEATH: i32 = 1;

    const FS_EXECUTE: u64 = 1 << 0;
    const FS_WRITE_FILE: u64 = 1 << 1;
    const FS_READ_FILE: u64 = 1 << 2;
    const FS_READ_DIR: u64 = 1 << 3;
    const FS_REMOVE_DIR: u64 = 1 << 4;
    const FS_REMOVE_FILE: u64 = 1 << 5;
    const FS_MAKE_CHAR: u64 = 1 << 6;
    const FS_MAKE_DIR: u64 = 1 << 7;
    const FS_MAKE_REG: u64 = 1 << 8;
    const FS_MAKE_SOCK: u64 = 1 << 9;
    const FS_MAKE_FIFO: u64 = 1 << 10;
    const FS_MAKE_BLOCK: u64 = 1 << 11;
    const FS_MAKE_SYM: u64 = 1 << 12;

    const HANDLED_FS_ABI1: u64 = FS_EXECUTE
        | FS_WRITE_FILE
        | FS_READ_FILE
        | FS_READ_DIR
        | FS_REMOVE_DIR
        | FS_REMOVE_FILE
        | FS_MAKE_CHAR
        | FS_MAKE_DIR
        | FS_MAKE_REG
        | FS_MAKE_SOCK
        | FS_MAKE_FIFO
        | FS_MAKE_BLOCK
        | FS_MAKE_SYM;

    const ALLOW_FS: u64 = FS_EXECUTE | FS_READ_FILE | FS_READ_DIR;

    #[repr(C)]
    struct RulesetAttr {
        handled_access_fs: u64,
    }

    #[repr(C)]
    struct PathBeneathAttr {
        allowed_access: u64,
        parent_fd: i32,
    }

    fn failed(msg: &str) -> Error {
        Error::new(
            ErrorKind::PermissionDenied,
            format!("{LANDLOCK_FAILED}: {msg}"),
        )
    }

    pub(super) fn restrict_fs_self(allow: &[PathBuf]) -> Result<(), Error> {
        if allow.is_empty() {
            return Err(failed("empty allowlist"));
        }
        let abi = unsafe {
            libc::syscall(
                libc::SYS_landlock_create_ruleset,
                ptr::null::<u8>(),
                0usize,
                LANDLOCK_CREATE_RULESET_VERSION,
            )
        };
        if abi < 1 {
            return Err(failed("kernel landlock ABI < 1"));
        }
        let attr = RulesetAttr {
            handled_access_fs: HANDLED_FS_ABI1,
        };
        let ruleset = unsafe {
            libc::syscall(
                libc::SYS_landlock_create_ruleset,
                &attr as *const RulesetAttr,
                std::mem::size_of::<RulesetAttr>(),
                0u32,
            )
        };
        if ruleset < 0 {
            return Err(failed("landlock_create_ruleset"));
        }
        let ruleset_fd = ruleset as i32;
        let mut added = 0u32;
        for path in allow {
            if add_path(ruleset_fd, path).is_ok() {
                added += 1;
            }
        }
        if added == 0 {
            unsafe {
                libc::close(ruleset_fd);
            }
            return Err(failed("no allowlist path could be added"));
        }
        let nnp = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if nnp != 0 {
            unsafe {
                libc::close(ruleset_fd);
            }
            return Err(failed("PR_SET_NO_NEW_PRIVS"));
        }
        let restrict = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, ruleset_fd, 0u32) };
        unsafe {
            libc::close(ruleset_fd);
        }
        if restrict != 0 {
            return Err(failed("landlock_restrict_self"));
        }
        Ok(())
    }

    fn add_path(ruleset_fd: i32, path: &Path) -> Result<(), Error> {
        let c_path = CString::new(path.as_os_str().as_bytes())
            .map_err(|_| failed("allow path contains NUL"))?;
        let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
        if fd < 0 {
            return Err(failed("open O_PATH"));
        }
        let beneath = PathBeneathAttr {
            allowed_access: ALLOW_FS,
            parent_fd: fd,
        };
        let rc = unsafe {
            libc::syscall(
                libc::SYS_landlock_add_rule,
                ruleset_fd,
                LANDLOCK_RULE_PATH_BENEATH,
                &beneath as *const PathBeneathAttr,
                0u32,
            )
        };
        unsafe {
            libc::close(fd);
        }
        if rc != 0 {
            return Err(failed("landlock_add_rule"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landlock_enabled_from_opt_in_only() {
        assert!(!landlock_enabled_from(None));
        assert!(!landlock_enabled_from(Some("")));
        assert!(!landlock_enabled_from(Some("0")));
        assert!(!landlock_enabled_from(Some("off")));
        assert!(landlock_enabled_from(Some("1")));
        assert!(landlock_enabled_from(Some("true")));
        assert!(landlock_enabled_from(Some("YES")));
    }
}
