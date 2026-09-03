//! Linux seccomp syscall filter for the generate-local child (QUEUE #226).
//!
//! Applied in the child after fork, before exec ([`std::os::unix::process::CommandExt::pre_exec`]),
//! after Landlock when both are enabled. Default-allow BPF with a deny-list of
//! network, ptrace, mount, module, and namespace syscalls. A denied syscall is
//! `SECCOMP_RET_KILL_PROCESS` (SIGSYS).
//!
//! Opt-in only. Missing kernel / failed filter → fail-closed (not unsandboxed
//! success). netns is a later atom.

use std::io::Error;
#[cfg(not(target_os = "linux"))]
use std::io::ErrorKind;

/// Fail-closed when the seccomp filter cannot be installed.
pub const SECCOMP_FAILED: &str =
    "generate process seccomp filter failed (fail-closed; not VERIFIED)";

/// Fail-closed when seccomp is requested on a non-Linux host.
pub const SECCOMP_UNSUPPORTED: &str =
    "generate process seccomp unsupported (fail-closed; not VERIFIED)";

/// Child was killed for a forbidden syscall (SIGSYS).
pub const SECCOMP_VIOLATION: &str =
    "generate process forbidden syscall (seccomp; fail-closed; not VERIFIED)";

/// `AIRA_LLM_SECCOMP=1|true|yes` enables the filter on [`super::ProcessBackend`].
pub const ENV_LLM_SECCOMP: &str = "AIRA_LLM_SECCOMP";

/// Parse the seccomp opt-in env value. Unset / anything else → off.
pub fn seccomp_enabled_from(value: Option<&str>) -> bool {
    match value {
        Some(v) if v.eq_ignore_ascii_case("1") => true,
        Some(v) if v.eq_ignore_ascii_case("true") => true,
        Some(v) if v.eq_ignore_ascii_case("yes") => true,
        _ => false,
    }
}

/// Install the deny-list filter on the current thread (child).
pub fn restrict_syscalls_self() -> Result<(), Error> {
    #[cfg(target_os = "linux")]
    {
        linux::restrict_syscalls_self()
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(Error::new(ErrorKind::Unsupported, SECCOMP_UNSUPPORTED))
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::SECCOMP_FAILED;
    use std::io::{Error, ErrorKind};

    /// linux/audit.h: EM | 64-bit | little-endian.
    #[cfg(target_arch = "x86_64")]
    const AUDIT_ARCH: u32 = 0xC000_003E;
    #[cfg(target_arch = "aarch64")]
    const AUDIT_ARCH: u32 = 0xC000_00B7;
    #[cfg(target_arch = "riscv64")]
    const AUDIT_ARCH: u32 = 0xC000_00F3;
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "riscv64"
    )))]
    const AUDIT_ARCH: u32 = 0;

    const OFF_NR: u32 = 0;
    const OFF_ARCH: u32 = 4;

    fn stmt(code: u32, k: u32) -> libc::sock_filter {
        libc::sock_filter {
            code: code as u16,
            jt: 0,
            jf: 0,
            k,
        }
    }

    fn jump(code: u32, k: u32, jt: u8, jf: u8) -> libc::sock_filter {
        libc::sock_filter {
            code: code as u16,
            jt,
            jf,
            k,
        }
    }

    fn failed(msg: &str) -> Error {
        Error::new(
            ErrorKind::PermissionDenied,
            format!("{SECCOMP_FAILED}: {msg}"),
        )
    }

    fn deny_nrs() -> Vec<u32> {
        [
            libc::SYS_socket,
            libc::SYS_socketpair,
            libc::SYS_bind,
            libc::SYS_connect,
            libc::SYS_listen,
            libc::SYS_accept,
            libc::SYS_accept4,
            libc::SYS_sendto,
            libc::SYS_recvfrom,
            libc::SYS_sendmsg,
            libc::SYS_recvmsg,
            libc::SYS_ptrace,
            libc::SYS_mount,
            libc::SYS_umount2,
            libc::SYS_pivot_root,
            libc::SYS_swapon,
            libc::SYS_swapoff,
            libc::SYS_init_module,
            libc::SYS_delete_module,
            libc::SYS_bpf,
            libc::SYS_unshare,
            libc::SYS_setns,
            libc::SYS_reboot,
            libc::SYS_kexec_load,
            libc::SYS_perf_event_open,
        ]
        .into_iter()
        .map(|n| n as u32)
        .collect()
    }

    pub(super) fn restrict_syscalls_self() -> Result<(), Error> {
        if AUDIT_ARCH == 0 {
            return Err(failed("unsupported architecture"));
        }
        let nnp = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if nnp != 0 {
            return Err(failed("PR_SET_NO_NEW_PRIVS"));
        }
        let ld_abs = libc::BPF_LD | libc::BPF_W | libc::BPF_ABS;
        let jmp_eq = libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K;
        let ret_k = libc::BPF_RET | libc::BPF_K;
        let kill = stmt(ret_k, libc::SECCOMP_RET_KILL_PROCESS);
        let allow = stmt(ret_k, libc::SECCOMP_RET_ALLOW);
        let mut filter = vec![
            stmt(ld_abs, OFF_ARCH),
            jump(jmp_eq, AUDIT_ARCH, 1, 0),
            kill,
            stmt(ld_abs, OFF_NR),
        ];
        for nr in deny_nrs() {
            filter.push(jump(jmp_eq, nr, 0, 1));
            filter.push(kill);
        }
        filter.push(allow);
        let mut prog = libc::sock_fprog {
            len: filter.len() as libc::c_ushort,
            filter: filter.as_mut_ptr(),
        };
        let rc = unsafe {
            libc::syscall(
                libc::SYS_seccomp,
                libc::SECCOMP_SET_MODE_FILTER,
                0u32,
                &mut prog as *mut libc::sock_fprog,
            )
        };
        if rc != 0 {
            return Err(failed("SECCOMP_SET_MODE_FILTER"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seccomp_enabled_from_opt_in_only() {
        assert!(!seccomp_enabled_from(None));
        assert!(!seccomp_enabled_from(Some("")));
        assert!(!seccomp_enabled_from(Some("0")));
        assert!(seccomp_enabled_from(Some("1")));
        assert!(seccomp_enabled_from(Some("true")));
        assert!(seccomp_enabled_from(Some("YES")));
    }
}
