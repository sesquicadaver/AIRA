//! Per-model verified/available lifecycle (`#344` / RFC-0227).
//!
//! Slot records under `models/verified/<slot>/pointer.json` and
//! `models/cache/<slot>/activated.json` survive when `*.latest.json` advances.
//! Latest pointers remain the default selection tip — not sole execution authority.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AcquisitionError;
use crate::types::{
    ActivatedPointer, ModelLifecycleEntry, VerifiedPointer, ACTIVATED_POINTER_REL,
    ACTIVATED_SLOT_POINTER_NAME, CACHE_REL, VERIFIED_POINTER_REL, VERIFIED_REL,
    VERIFIED_SLOT_POINTER_NAME,
};
use crate::util::sanitize_slot;

/// Absolute path to the per-model verified slot pointer.
pub fn verified_slot_pointer_path(aira_root: impl AsRef<Path>, model_ref: &str) -> PathBuf {
    aira_root
        .as_ref()
        .join(VERIFIED_REL)
        .join(sanitize_slot(model_ref))
        .join(VERIFIED_SLOT_POINTER_NAME)
}

/// Absolute path to the per-model activated slot pointer.
pub fn activated_slot_pointer_path(aira_root: impl AsRef<Path>, model_ref: &str) -> PathBuf {
    aira_root
        .as_ref()
        .join(CACHE_REL)
        .join(sanitize_slot(model_ref))
        .join(ACTIVATED_SLOT_POINTER_NAME)
}

/// Persist a verified slot locator (does not clear other models).
pub(crate) fn write_verified_slot(
    root: &Path,
    pointer: &VerifiedPointer,
) -> Result<(), AcquisitionError> {
    let path = verified_slot_pointer_path(root, &pointer.model_ref);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;
    Ok(())
}

/// Persist an activated slot locator (does not clear other models).
pub(crate) fn write_activated_slot(
    root: &Path,
    pointer: &ActivatedPointer,
) -> Result<(), AcquisitionError> {
    let path = activated_slot_pointer_path(root, &pointer.model_ref);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;
    Ok(())
}

/// Load the verified slot pointer for `model_ref` (independent of `verified.latest`).
pub fn load_verified_slot(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
) -> Result<VerifiedPointer, AcquisitionError> {
    let root = aira_root.as_ref();
    let path = verified_slot_pointer_path(root, model_ref);
    if path.is_file() {
        return read_verified_pointer(&path);
    }
    // Compat: pre-#344 roots only had verified.latest.
    let latest_path = root.join(VERIFIED_POINTER_REL);
    if latest_path.is_file() {
        let latest = read_verified_pointer(&latest_path)?;
        if latest.model_ref == model_ref {
            return Ok(latest);
        }
    }
    Err(AcquisitionError::NoVerified)
}

/// Load the activated slot pointer for `model_ref` (independent of `activated.latest`).
pub fn load_activated_slot(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
) -> Result<ActivatedPointer, AcquisitionError> {
    let root = aira_root.as_ref();
    let path = activated_slot_pointer_path(root, model_ref);
    if path.is_file() {
        return read_activated_pointer(&path);
    }
    let latest_path = root.join(ACTIVATED_POINTER_REL);
    if latest_path.is_file() {
        let latest = read_activated_pointer(&latest_path)?;
        if latest.model_ref == model_ref {
            return Ok(latest);
        }
    }
    Err(AcquisitionError::NoActivated)
}

/// List independent verified/available lifecycle entries after cold restart.
pub fn list_model_lifecycle(
    aira_root: impl AsRef<Path>,
) -> Result<Vec<ModelLifecycleEntry>, AcquisitionError> {
    let root = aira_root.as_ref();
    let mut by_ref: BTreeMap<String, ModelLifecycleEntry> = BTreeMap::new();

    let verified_root = root.join(VERIFIED_REL);
    if verified_root.is_dir() {
        for entry in
            fs::read_dir(&verified_root).map_err(|e| AcquisitionError::Io(e.to_string()))?
        {
            let entry = entry.map_err(|e| AcquisitionError::Io(e.to_string()))?;
            let pointer_path = entry.path().join(VERIFIED_SLOT_POINTER_NAME);
            if !pointer_path.is_file() {
                continue;
            }
            let pointer = read_verified_pointer(&pointer_path)?;
            let e = by_ref
                .entry(pointer.model_ref.clone())
                .or_insert_with(|| empty_entry(&pointer.model_ref));
            e.verified = true;
            e.verified_path = Some(pointer.verified_path);
            if e.content_hash.is_none() {
                e.content_hash = Some(pointer.content_hash);
            }
        }
    }

    let cache_root = root.join(CACHE_REL);
    if cache_root.is_dir() {
        for entry in fs::read_dir(&cache_root).map_err(|e| AcquisitionError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| AcquisitionError::Io(e.to_string()))?;
            let pointer_path = entry.path().join(ACTIVATED_SLOT_POINTER_NAME);
            if !pointer_path.is_file() {
                continue;
            }
            let pointer = read_activated_pointer(&pointer_path)?;
            let e = by_ref
                .entry(pointer.model_ref.clone())
                .or_insert_with(|| empty_entry(&pointer.model_ref));
            e.available = true;
            e.cache_path = Some(pointer.cache_path);
            e.content_hash = Some(pointer.content_hash);
            if e.verified_path.is_none() {
                e.verified_path = Some(pointer.verified_path);
            }
            // Pack B / audit #5: activated ≠ weight-verified. Host-ollama bind
            // markers stay verified=false unless a verified pointer was seen.
            // Do not force e.verified = true here.
        }
    }

    Ok(by_ref.into_values().collect())
}

fn empty_entry(model_ref: &str) -> ModelLifecycleEntry {
    ModelLifecycleEntry {
        model_ref: model_ref.to_string(),
        verified: false,
        available: false,
        content_hash: None,
        verified_path: None,
        cache_path: None,
    }
}

fn read_verified_pointer(path: &Path) -> Result<VerifiedPointer, AcquisitionError> {
    let raw = fs::read_to_string(path).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| AcquisitionError::Other(e.to_string()))
}

fn read_activated_pointer(path: &Path) -> Result<ActivatedPointer, AcquisitionError> {
    let raw = fs::read_to_string(path).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| AcquisitionError::Other(e.to_string()))
}
