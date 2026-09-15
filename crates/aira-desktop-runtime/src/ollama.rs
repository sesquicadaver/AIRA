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
/// Fail-closed when the binary is missing or exits non-zero.
/// Does **not** talk to `aira-node` and does not mutate activation.
pub fn list_ollama_models(bin: impl AsRef<Path>) -> Result<Vec<OllamaListEntry>> {
    let bin = bin.as_ref();
    let output = Command::new(bin)
        .arg("list")
        .output()
        .with_context(|| format!("spawn `{} list` (is ollama installed?)", bin.display()))?;
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
}
