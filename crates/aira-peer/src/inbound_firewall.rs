//! Inbound firewall / netns honesty helpers (QUEUE `#252` / Phase N-fix).
//!
//! Distinguishes:
//! - [`crate::nat_relay::INBOUND_BLOCK_KIND_NON_LISTENING_PLACEHOLDER`] — `#246`
//! - [`INBOUND_BLOCK_KIND_FIREWALL_DROP`] — listening socket + INPUT DROP
//!
//! Unprivileged hosts often cannot create netns / install iptables. CI then
//! takes the **documented** path: operator script
//! [`scripts/inbound_firewall_smoke.sh`](../../../scripts/inbound_firewall_smoke.sh)
//! (Docker + `NET_ADMIN`) proves listening ≠ non-listening.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Honest semantics: socket is listening; inbound refused by firewall DROP.
pub const INBOUND_BLOCK_KIND_FIREWALL_DROP: &str = "firewall-input-drop";

/// Relative path from repo root to the Docker/iptables smoke script.
pub const INBOUND_FIREWALL_SMOKE_SCRIPT: &str = "scripts/inbound_firewall_smoke.sh";

/// Locate repository root from this crate's `CARGO_MANIFEST_DIR`.
pub fn repo_root_from_peer_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Path to the inbound-firewall smoke script.
pub fn inbound_firewall_smoke_script(repo_root: impl AsRef<Path>) -> PathBuf {
    repo_root.as_ref().join(INBOUND_FIREWALL_SMOKE_SCRIPT)
}

/// True when `docker` is on PATH and the daemon answers `docker info`.
pub fn docker_daemon_usable() -> bool {
    Command::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run the Docker + iptables smoke (listening → connect OK → DROP → connect fail).
///
/// Returns `Ok(stdout)` on success. Callers without Docker should take the
/// documented-path branch instead of treating this as a hard failure.
pub fn run_inbound_firewall_smoke(repo_root: impl AsRef<Path>) -> Result<String, String> {
    let script = inbound_firewall_smoke_script(&repo_root);
    if !script.is_file() {
        return Err(format!("missing smoke script {}", script.display()));
    }
    if !docker_daemon_usable() {
        return Err("docker daemon not usable".into());
    }
    let output = Command::new("bash")
        .arg(&script)
        .current_dir(repo_root.as_ref())
        .output()
        .map_err(|e| format!("spawn smoke script: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(format!(
            "smoke failed status={:?}\nstdout={stdout}\nstderr={stderr}",
            output.status
        ));
    }
    for needle in [
        "LISTEN_OK",
        "CONTROL_CONNECT_OK",
        "FIREWALL_BLOCK_OK",
        "STILL_LISTENING_OK",
        INBOUND_BLOCK_KIND_FIREWALL_DROP,
    ] {
        if !stdout.contains(needle) {
            return Err(format!(
                "smoke stdout missing {needle}:\n{stdout}\n{stderr}"
            ));
        }
    }
    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    use crate::nat_relay::INBOUND_BLOCK_KIND_NON_LISTENING_PLACEHOLDER;

    #[test]
    fn firewall_kind_differs_from_non_listening_placeholder() {
        assert_ne!(
            INBOUND_BLOCK_KIND_FIREWALL_DROP,
            INBOUND_BLOCK_KIND_NON_LISTENING_PLACEHOLDER
        );
        assert!(INBOUND_BLOCK_KIND_FIREWALL_DROP.contains("firewall"));
        assert!(INBOUND_BLOCK_KIND_NON_LISTENING_PLACEHOLDER.contains("non-listening"));
    }

    #[test]
    fn listening_socket_is_not_the_same_as_non_listening_placeholder() {
        // Control: a real listener accepts a connect (≠ non-listening placeholder).
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let accept = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            drop(stream);
        });
        let ok = std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(2));
        assert!(ok.is_ok(), "listening socket must accept connect");
        accept.join().unwrap();

        // After close: same as #246 placeholder — connect refused.
        let dead = format!("127.0.0.1:{}", addr.port());
        // Port may be reused; pick a known non-listening prime instead.
        let placeholder = "127.0.0.1:49169";
        let err = std::net::TcpStream::connect_timeout(
            &placeholder.parse().unwrap(),
            Duration::from_millis(300),
        );
        assert!(
            err.is_err(),
            "non-listening placeholder {placeholder} must refuse connect"
        );
        let _ = dead;
    }

    #[test]
    fn documented_smoke_script_exists() {
        let root = repo_root_from_peer_crate();
        let script = inbound_firewall_smoke_script(&root);
        assert!(
            script.is_file(),
            "documented path requires {}",
            script.display()
        );
        let text = std::fs::read_to_string(&script).unwrap();
        assert!(text.contains("NET_ADMIN") || text.contains("iptables"));
        assert!(text.contains(INBOUND_BLOCK_KIND_FIREWALL_DROP));
    }
}
