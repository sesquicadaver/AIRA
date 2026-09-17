//! Host `ollama list` probe for Desktop Settings (process backend bind).
//!
//! Listing models is **observe-only** on the Desktop host. It does not activate
//! Phase D weights, does not grant VERIFIED, and does not imply the running
//! `aira-node` already has `AIRA_LLM_BACKEND=process` until restart applies
//! settings.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

/// Default program when Settings leave `llm_process_bin` empty.
pub const DEFAULT_OLLAMA_BIN: &str = "ollama";

/// One row from `ollama list` (name column only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaListEntry {
    pub name: String,
}

/// Parse tabular `ollama list` stdout (header + whitespace columns).
///
/// First column is the model name (`name:tag` or `org/name:tag`). Empty /
/// header-only output → empty vec (not an error).
pub fn parse_ollama_list_stdout(stdout: &str) -> Vec<OllamaListEntry> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut cols = line.split_whitespace();
        let Some(name) = cols.next() else {
            continue;
        };
        if name.eq_ignore_ascii_case("NAME") {
            continue;
        }
        out.push(OllamaListEntry {
            name: name.to_string(),
        });
    }
    out
}

/// Resolve program path or PATH name (existence not required until spawn).
pub fn resolve_ollama_bin(explicit: Option<&str>) -> PathBuf {
    match explicit.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => PathBuf::from(s),
        None => PathBuf::from(DEFAULT_OLLAMA_BIN),
    }
}

/// Run `ollama list` on the Desktop host and return model names.
///
/// Fail-closed when the binary is missing, exits non-zero, or exceeds timeout
/// (Pack D / audit #3 — no unbounded UI hang). On timeout the child is killed
/// and waited so hang probes do not accumulate orphans.
pub fn list_ollama_models(bin: impl AsRef<Path>) -> Result<Vec<OllamaListEntry>> {
    list_ollama_models_with_timeout(bin, Duration::from_secs(15))
}

/// Same as [`list_ollama_models`] with an explicit wait bound.
pub fn list_ollama_models_with_timeout(
    bin: impl AsRef<Path>,
    timeout: Duration,
) -> Result<Vec<OllamaListEntry>> {
    let bin = bin.as_ref();
    let mut child = Command::new(bin)
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn `{} list` (is ollama installed?)", bin.display()))?;

    let start = Instant::now();
    loop {
        match child
            .try_wait()
            .with_context(|| format!("wait `{} list`", bin.display()))?
        {
            Some(status) => {
                let mut stdout = String::new();
                let mut stderr = String::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_string(&mut stdout);
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_string(&mut stderr);
                }
                if !status.success() {
                    bail!(
                        "`{} list` failed (status {:?}): {}",
                        bin.display(),
                        status.code(),
                        stderr.trim()
                    );
                }
                return Ok(parse_ollama_list_stdout(&stdout));
            }
            None => {
                if start.elapsed() >= timeout {
                    let child_pid = child.id();
                    let _ = child.kill();
                    let _ = child.wait();
                    bail!(
                        "`ollama list` timed out after {}ms (fail-closed; killed pid {child_pid}; UI not blocked forever)",
                        timeout.as_millis()
                    );
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn parse_skips_header_and_keeps_names() {
        let raw = "\
NAME                                                                 ID              SIZE      MODIFIED
llama3:latest                                                        365c0bd3c000    4.7 GB    7 hours ago
koill/sentence-transformers:paraphrase-multilingual-minilm-l12-v2    3ee258ffc9f2    476 MB    7 hours ago
";
        let rows = parse_ollama_list_stdout(raw);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "llama3:latest");
        assert_eq!(
            rows[1].name,
            "koill/sentence-transformers:paraphrase-multilingual-minilm-l12-v2"
        );
    }

    #[test]
    fn parse_empty_is_empty() {
        assert!(parse_ollama_list_stdout("").is_empty());
        assert!(parse_ollama_list_stdout("NAME ID SIZE MODIFIED\n").is_empty());
    }

    #[test]
    fn resolve_default_bin() {
        assert_eq!(resolve_ollama_bin(None), PathBuf::from(DEFAULT_OLLAMA_BIN));
        assert_eq!(
            resolve_ollama_bin(Some("  /usr/local/bin/ollama ")),
            PathBuf::from("/usr/local/bin/ollama")
        );
    }

    /// Pack D / P2: hang script must fail-closed via kill+wait (no orphan sleep).
    #[cfg(unix)]
    #[test]
    fn list_ollama_models_timeout_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let pidfile = dir.path().join("hang.pid");
        let script = dir.path().join("hang-ollama.sh");
        let pidfile_disp = pidfile.display().to_string();
        std::fs::write(
            &script,
            format!("#!/bin/sh\necho $$ > '{pidfile_disp}'\nexec sleep 30\n"),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).unwrap();
        let err = list_ollama_models_with_timeout(&script, Duration::from_millis(200))
            .expect_err("hang must timeout");
        assert!(
            err.to_string().contains("timed out"),
            "expected timeout, got {err:#}"
        );
        // Child must be dead — no orphan accumulation across CI runs.
        let pid_text = std::fs::read_to_string(&pidfile).unwrap_or_default();
        let pid: u32 = pid_text.trim().parse().expect("hang script wrote pid");
        assert!(
            !crate::process::pid_alive(pid),
            "hang child pid {pid} must be killed+waited after list timeout"
        );
    }
}
