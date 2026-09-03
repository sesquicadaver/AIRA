//! Sandbox-required policy for generate-local (QUEUE #228).
//!
//! Opt-in Landlock / seccomp / netns already fail-close when **requested** and
//! the kernel cannot apply them. This module is the operator policy: when
//! sandbox is **required**, a missing kernel or non-Linux OS MUST be
//! CapsuleFailed — never unsandboxed success / fake VERIFIED. ollama-style
//! loopback cannot satisfy a required OS sandbox (netns would isolate host
//! `127.0.0.1`); that is fail-closed too.

/// Fail-closed when OS sandbox is required but the kernel or OS cannot provide it.
pub const SANDBOX_REQUIRED: &str =
    "generate process sandbox required but unavailable (fail-closed; not VERIFIED)";

/// Fail-closed when OS sandbox is required for an ollama-style loopback child.
pub const SANDBOX_REQUIRED_LOOPBACK: &str =
    "generate process sandbox required cannot isolate ollama loopback (fail-closed; not VERIFIED)";

/// `AIRA_LLM_SANDBOX_REQUIRED=1|true|yes` requires OS sandbox on [`super::ProcessBackend`].
pub const ENV_LLM_SANDBOX_REQUIRED: &str = "AIRA_LLM_SANDBOX_REQUIRED";

/// Parse the sandbox-required env value. Unset / anything else → off.
pub fn sandbox_required_from(value: Option<&str>) -> bool {
    match value {
        Some(v) if v.eq_ignore_ascii_case("1") => true,
        Some(v) if v.eq_ignore_ascii_case("true") => true,
        Some(v) if v.eq_ignore_ascii_case("yes") => true,
        _ => false,
    }
}

/// Enforce sandbox-required: loopback exception, non-Linux, or missing kernel.
pub fn enforce(host_loopback: bool, kernel_ok: bool) -> Result<(), String> {
    if host_loopback {
        return Err(SANDBOX_REQUIRED_LOOPBACK.into());
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = kernel_ok;
        return Err(SANDBOX_REQUIRED.into());
    }
    #[cfg(target_os = "linux")]
    {
        if !kernel_ok {
            return Err(SANDBOX_REQUIRED.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_required_from_opt_in_only() {
        assert!(!sandbox_required_from(None));
        assert!(!sandbox_required_from(Some("")));
        assert!(!sandbox_required_from(Some("0")));
        assert!(sandbox_required_from(Some("1")));
        assert!(sandbox_required_from(Some("true")));
        assert!(sandbox_required_from(Some("YES")));
    }

    #[test]
    fn enforce_missing_kernel_is_fail_closed() {
        let err = enforce(false, false).unwrap_err();
        assert!(err.contains(SANDBOX_REQUIRED), "got {err}");
        assert!(!err.contains("VERIFIED result"));
    }

    #[test]
    fn enforce_ollama_loopback_is_fail_closed() {
        let err = enforce(true, true).unwrap_err();
        assert!(err.contains(SANDBOX_REQUIRED_LOOPBACK), "got {err}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn enforce_linux_kernel_ok_is_ready() {
        enforce(false, true).expect("linux + kernel must be ready");
    }
}
