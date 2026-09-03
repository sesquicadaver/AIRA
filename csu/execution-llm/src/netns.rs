//! Linux network namespace for the generate-local child (QUEUE #227).
//!
//! Applied in the child after fork, before exec — **before** Landlock and seccomp
//! (`unshare` is later denied by the `#226` filter). Unprivileged path: user
//! namespace + uid/gid map, then `CLONE_NEWNET`. The child's `127.0.0.1` is not
//! the host loopback, so ollama-style daemons would break; [`ProcessBackend::ollama`]
//! combined with netns fail-closes instead of applying isolation silently.
//!
//! Opt-in only. Missing kernel / EPERM / failed unshare → fail-closed.

use std::io::Error;
#[cfg(not(target_os = "linux"))]
use std::io::ErrorKind;

/// Fail-closed when the network namespace cannot be applied.
pub const NETNS_FAILED: &str = "generate process netns restrict failed (fail-closed; not VERIFIED)";

/// Fail-closed when netns is requested on a non-Linux host.
pub const NETNS_UNSUPPORTED: &str =
    "generate process netns unsupported (fail-closed; not VERIFIED)";

/// Fail-closed when netns would isolate host loopback used by ollama-style children.
pub const NETNS_BLOCKS_LOOPBACK: &str =
    "generate process netns would isolate host loopback (ollama exception; fail-closed; not VERIFIED)";

/// `AIRA_LLM_NETNS=1|true|yes` enables netns on [`super::ProcessBackend`] (offline argv).
pub const ENV_LLM_NETNS: &str = "AIRA_LLM_NETNS";

/// Parse the netns opt-in env value. Unset / anything else → off.
pub fn netns_enabled_from(value: Option<&str>) -> bool {
    match value {
        Some(v) if v.eq_ignore_ascii_case("1") => true,
        Some(v) if v.eq_ignore_ascii_case("true") => true,
        Some(v) if v.eq_ignore_ascii_case("yes") => true,
        _ => false,
    }
}

/// Enter a new network namespace on the current thread (child).
pub fn restrict_netns_self() -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        linux::restrict_netns_self()
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(Error::new(ErrorKind::Unsupported, NETNS_UNSUPPORTED))
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::NETNS_FAILED;
    use std::io::{Error, ErrorKind};

    fn failed(msg: &str) -> Error {
        Error::new(
            ErrorKind::PermissionDenied,
            format!("{NETNS_FAILED}: {msg}"),
        )
    }

    fn write_proc(path: &str, data: &str) -> Result<(), Error> {
        std::fs::write(path, data).map_err(|e| failed(&format!("{path}: {e}")))
    }

    pub(super) fn restrict_netns_self() -> Result<(), Error> {
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        let rc = unsafe { libc::unshare(libc::CLONE_NEWUSER) };
        if rc != 0 {
            return Err(failed("unshare CLONE_NEWUSER"));
        }
        write_proc("/proc/self/setgroups", "deny\n")?;
        write_proc("/proc/self/uid_map", &format!("0 {uid} 1\n"))?;
        write_proc("/proc/self/gid_map", &format!("0 {gid} 1\n"))?;
        let rc = unsafe { libc::unshare(libc::CLONE_NEWNET) };
        if rc != 0 {
            return Err(failed("unshare CLONE_NEWNET"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn netns_enabled_from_opt_in_only() {
        assert!(!netns_enabled_from(None));
        assert!(!netns_enabled_from(Some("")));
        assert!(!netns_enabled_from(Some("0")));
        assert!(netns_enabled_from(Some("1")));
        assert!(netns_enabled_from(Some("true")));
        assert!(netns_enabled_from(Some("YES")));
    }
}
