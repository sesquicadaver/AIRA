//! Durable reuse-index catalog (`problems/reuse-index.json`, RFC-0087 / `#204`).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use aira_object::ContentHash;
use serde::{Deserialize, Serialize};

/// Persistent map: problem-text `sha256:` hash → reusable artifact id.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ReuseIndex {
    pub by_content_hash: BTreeMap<String, String>,
}

pub(crate) fn problem_text_hash(text: &str) -> String {
    ContentHash::sha256_bytes(text.as_bytes())
        .as_str()
        .to_string()
}

pub(crate) fn load_reuse_index(path: &Path) -> Result<ReuseIndex, String> {
    if !path.exists() {
        return Ok(ReuseIndex::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub(crate) fn lookup_artifact_id(path: &Path, text: &str) -> Result<Option<String>, String> {
    let idx = load_reuse_index(path)?;
    Ok(idx.by_content_hash.get(&problem_text_hash(text)).cloned())
}
