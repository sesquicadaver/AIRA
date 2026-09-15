//! Model select API Auto / required (`#345` / RFC-0228).
//!
//! Resolves a concrete `model_ref` from per-model lifecycle (#344) without GUI.
//! Does **not** grant VERIFIED; callers place the result into admission `model_ref`.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::lifecycle::list_model_lifecycle;
use crate::types::{ActivatedPointer, ACTIVATED_POINTER_REL};

/// How the caller requests a local model for generate-local.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelSelection {
    /// Prefer `activated.latest` when that model is available; else first available by `model_ref`.
    Auto,
    /// Exact model required — fail-closed if unready or removed.
    Required(String),
}

/// Successful select result (ready for admission `model_ref` / gate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedModel {
    pub model_ref: String,
    pub content_hash: Option<String>,
    pub cache_path: Option<String>,
    /// Whether Auto tip preference or an explicit Required resolved this entry.
    pub via: ModelSelectionVia,
}

/// How [`select_model`] chose the concrete ref.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelSelectionVia {
    /// Explicit [`ModelSelection::Required`].
    Required,
    /// Auto used `activated.latest` because that model is available.
    AutoLatestTip,
    /// Auto fell back to deterministic first available (`model_ref` order).
    AutoFirstAvailable,
}

/// Explained select failures (`#345` / RFC-0228) — no silent fallback.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModelSelectError {
    #[error("no available local models for Auto select (fail-closed)")]
    NoAvailableModels,
    #[error(
        "required model {model_ref} is not ready (verified={verified}, available={available}; fail-closed)"
    )]
    ModelUnready {
        model_ref: String,
        verified: bool,
        available: bool,
    },
    #[error("required model {model_ref} is removed or unknown (fail-closed)")]
    ModelRemoved { model_ref: String },
    #[error("model select io: {0}")]
    Io(String),
}

/// Resolve Auto or Required selection against durable lifecycle slots.
///
/// `#345` / RFC-0228: unready/removed → explained [`ModelSelectError`]; never silent substitute.
pub fn select_model(
    aira_root: impl AsRef<Path>,
    selection: ModelSelection,
) -> Result<SelectedModel, ModelSelectError> {
    let root = aira_root.as_ref();
    let life = list_model_lifecycle(root).map_err(|e| ModelSelectError::Io(e.to_string()))?;

    match selection {
        ModelSelection::Auto => {
            let available: Vec<_> = life.into_iter().filter(|e| e.available).collect();
            if available.is_empty() {
                return Err(ModelSelectError::NoAvailableModels);
            }
            if let Some(tip) = read_latest_tip(root)? {
                if let Some(hit) = available.iter().find(|e| e.model_ref == tip.model_ref) {
                    return Ok(SelectedModel {
                        model_ref: hit.model_ref.clone(),
                        content_hash: hit.content_hash.clone(),
                        cache_path: hit.cache_path.clone(),
                        via: ModelSelectionVia::AutoLatestTip,
                    });
                }
            }
            // Deterministic among A/B when tip missing or tip not available.
            let first = &available[0];
            Ok(SelectedModel {
                model_ref: first.model_ref.clone(),
                content_hash: first.content_hash.clone(),
                cache_path: first.cache_path.clone(),
                via: ModelSelectionVia::AutoFirstAvailable,
            })
        }
        ModelSelection::Required(want) => {
            let Some(entry) = life.iter().find(|e| e.model_ref == want) else {
                return Err(ModelSelectError::ModelRemoved { model_ref: want });
            };
            if !entry.available {
                return Err(ModelSelectError::ModelUnready {
                    model_ref: want,
                    verified: entry.verified,
                    available: false,
                });
            }
            Ok(SelectedModel {
                model_ref: entry.model_ref.clone(),
                content_hash: entry.content_hash.clone(),
                cache_path: entry.cache_path.clone(),
                via: ModelSelectionVia::Required,
            })
        }
    }
}

fn read_latest_tip(root: &Path) -> Result<Option<ActivatedPointer>, ModelSelectError> {
    let path = root.join(ACTIVATED_POINTER_REL);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| ModelSelectError::Io(e.to_string()))?;
    let tip: ActivatedPointer =
        serde_json::from_str(&raw).map_err(|e| ModelSelectError::Io(e.to_string()))?;
    Ok(Some(tip))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ModelLifecycleEntry;
    use std::fs;

    fn write_slot_activated(root: &Path, model_ref: &str, bytes_tag: &str) {
        let slot = crate::util::sanitize_slot(model_ref);
        let cache_dir = root.join("models/cache").join(&slot);
        fs::create_dir_all(&cache_dir).unwrap();
        let weights = cache_dir.join("weights.bin");
        fs::write(&weights, bytes_tag.as_bytes()).unwrap();
        let hash = aira_object::ContentHash::sha256_bytes(bytes_tag.as_bytes());
        let pointer = ActivatedPointer {
            updated_at: "2026-09-15T00:00:00Z".into(),
            model_ref: model_ref.into(),
            cache_path: weights.display().to_string(),
            verified_path: format!("models/verified/{slot}/weights.bin"),
            content_hash: hash.as_str().to_string(),
            evidence_artifact_id: format!("aira:artifact:acq-activate:{bytes_tag}"),
        };
        crate::lifecycle::write_activated_slot(root, &pointer).unwrap();
    }

    fn write_verified_only(root: &Path, model_ref: &str) {
        use crate::types::VerifiedPointer;
        let slot = crate::util::sanitize_slot(model_ref);
        let vdir = root.join("models/verified").join(&slot);
        fs::create_dir_all(&vdir).unwrap();
        let vpath = vdir.join("weights.bin");
        fs::write(&vpath, b"verified-only").unwrap();
        let hash = aira_object::ContentHash::sha256_bytes(b"verified-only");
        let pointer = VerifiedPointer {
            updated_at: "2026-09-15T00:00:00Z".into(),
            model_ref: model_ref.into(),
            verified_path: vpath.display().to_string(),
            quarantine_path: "models/quarantine/x".into(),
            content_hash: hash.as_str().to_string(),
            evidence_artifact_id: "aira:artifact:acq-verify-ok:x".into(),
        };
        crate::lifecycle::write_verified_slot(root, &pointer).unwrap();
    }

    #[test]
    fn auto_empty_is_explained() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let err = select_model(dir.path(), ModelSelection::Auto).unwrap_err();
        assert_eq!(err, ModelSelectError::NoAvailableModels);
        assert!(err.to_string().contains("no available"));
    }

    #[test]
    fn auto_prefers_latest_tip_when_available() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_slot_activated(root, "aira:model:alpha", "alpha");
        write_slot_activated(root, "aira:model:beta", "beta");
        // Tip = beta (not lex-first).
        let tip = ActivatedPointer {
            updated_at: "2026-09-15T00:00:00Z".into(),
            model_ref: "aira:model:beta".into(),
            cache_path: activated_slot_pointer_path_display(root, "aira:model:beta"),
            verified_path: "models/verified/x".into(),
            content_hash: "sha256:bb".into(),
            evidence_artifact_id: "aira:artifact:acq-activate:tip".into(),
        };
        fs::write(
            root.join(ACTIVATED_POINTER_REL),
            serde_json::to_string_pretty(&tip).unwrap(),
        )
        .unwrap();
        let sel = select_model(root, ModelSelection::Auto).unwrap();
        assert_eq!(sel.model_ref, "aira:model:beta");
        assert_eq!(sel.via, ModelSelectionVia::AutoLatestTip);
    }

    #[test]
    fn auto_falls_back_to_first_available_when_tip_missing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_slot_activated(root, "aira:model:zeta", "z");
        write_slot_activated(root, "aira:model:alpha", "a");
        let sel = select_model(root, ModelSelection::Auto).unwrap();
        assert_eq!(sel.model_ref, "aira:model:alpha");
        assert_eq!(sel.via, ModelSelectionVia::AutoFirstAvailable);
    }

    #[test]
    fn required_available_ok() {
        let dir = tempfile::tempdir().unwrap();
        write_slot_activated(dir.path(), "aira:model:need", "need");
        let sel = select_model(
            dir.path(),
            ModelSelection::Required("aira:model:need".into()),
        )
        .unwrap();
        assert_eq!(sel.model_ref, "aira:model:need");
        assert_eq!(sel.via, ModelSelectionVia::Required);
    }

    #[test]
    fn required_unready_is_explained() {
        let dir = tempfile::tempdir().unwrap();
        write_verified_only(dir.path(), "aira:model:half");
        let err = select_model(
            dir.path(),
            ModelSelection::Required("aira:model:half".into()),
        )
        .unwrap_err();
        match &err {
            ModelSelectError::ModelUnready {
                model_ref,
                verified,
                available,
            } => {
                assert_eq!(model_ref, "aira:model:half");
                assert!(*verified);
                assert!(!*available);
            }
            other => panic!("expected ModelUnready, got {other:?}"),
        }
        assert!(err.to_string().contains("not ready"));
    }

    #[test]
    fn required_removed_is_explained() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("models")).unwrap();
        let err = select_model(
            dir.path(),
            ModelSelection::Required("aira:model:gone".into()),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ModelSelectError::ModelRemoved {
                model_ref: "aira:model:gone".into()
            }
        );
        assert!(err.to_string().contains("removed") || err.to_string().contains("unknown"));
    }

    #[test]
    fn required_does_not_silent_substitute_other_available() {
        let dir = tempfile::tempdir().unwrap();
        write_slot_activated(dir.path(), "aira:model:other", "o");
        let err = select_model(
            dir.path(),
            ModelSelection::Required("aira:model:missing".into()),
        )
        .unwrap_err();
        assert!(matches!(err, ModelSelectError::ModelRemoved { .. }));
        // Lifecycle still has the other model — select must not return it.
        let life = list_model_lifecycle(dir.path()).unwrap();
        assert!(life.iter().any(|e: &ModelLifecycleEntry| e.available));
    }

    fn activated_slot_pointer_path_display(root: &Path, model_ref: &str) -> String {
        crate::lifecycle::activated_slot_pointer_path(root, model_ref)
            .with_file_name("weights.bin")
            .display()
            .to_string()
    }
}
