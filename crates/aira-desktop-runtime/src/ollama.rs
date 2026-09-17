//! Host `ollama list` probe for Desktop Settings (process backend bind).
//!
//! Listing models is **observe-only** on the Desktop host. It does not activate
//! Phase D weights, does not grant VERIFIED, and does not imply the running
//! `aira-node` already has `AIRA_LLM_BACKEND=process` until restart applies
//! settings.

use std::path::{Path, PathBuf};
use std::process::Command;

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
/// (Pack D / audit #3 — no unbounded UI hang).
pub fn list_ollama_models(bin: impl AsRef<Path>) -> Result<Vec<OllamaListEntry>> {
    list_ollama_models_with_timeout(bin, std::time::Duration::from_secs(15))
}

/// Same as [`list_ollama_models`] with an explicit wait bound.
pub fn list_ollama_models_with_timeout(
    bin: impl AsRef<Path>,
    timeout: std::time::Duration,
) -> Result<Vec<OllamaListEntry>> {
    use std::sync::mpsc;
    use std::thread;

    let bin = bin.as_ref().to_path_buf();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = (|| {
            let output = Command::new(&bin).arg("list").output().with_context(|| {
                format!("spawn `{} list` (is ollama installed?)", bin.display())
            })?;
            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                bail!(
                    "`{} list` failed (status {:?}): {}",
                    bin.display(),
                    output.status.code(),
                    err.trim()
                );
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(parse_ollama_list_stdout(&stdout))
        })();
        let _ = tx.send(result);
    });
    match rx.recv_timeout(timeout) {
        Ok(r) => r,
        Err(mpsc::RecvTimeoutError::Timeout) => bail!(
            "`ollama list` timed out after {}s (fail-closed; UI not blocked forever)",
            timeout.as_secs()
        ),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            bail!("`ollama list` worker disconnected (fail-closed)")
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

    /// Pack D: hang script must fail-closed via timeout (UI never waits forever).
    #[cfg(unix)]
    #[test]
    fn list_ollama_models_timeout_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("hang-ollama.sh");
        std::fs::write(&script, "#!/bin/sh\nsleep 30\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).unwrap();
        let err = list_ollama_models_with_timeout(&script, std::time::Duration::from_millis(200))
            .expect_err("hang must timeout");
        assert!(
            err.to_string().contains("timed out"),
            "expected timeout, got {err:#}"
        );
    }
}
