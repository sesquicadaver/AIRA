//! Content-addressed artifact store with supersession metadata.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use aira_object::{AiraRef, ContentHash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::descriptor::ArtifactDescriptor;

/// Artifact store errors.
#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("artifact immutable: {0}")]
    Immutable(AiraRef),
    #[error("artifact not found: {0}")]
    NotFound(AiraRef),
    #[error("unsigned artifact: {0}")]
    Unsigned(AiraRef),
    #[error("invalid artifact signature: {0}")]
    InvalidSignature(AiraRef),
    #[error("private artifact access denied: {0}")]
    AccessDenied(AiraRef),
    #[error("storage error: {0}")]
    Storage(String),
}

/// Policy ref that marks an artifact as private (default-deny on resolve).
pub const PRIVATE_ARTIFACT_POLICY: &str = "aira:policy:private";

/// True when descriptor is marked private via policy_refs.
pub fn is_private_artifact(descriptor: &ArtifactDescriptor) -> bool {
    descriptor
        .policy_refs
        .iter()
        .any(|r| r.as_str() == PRIVATE_ARTIFACT_POLICY)
}

/// Admit an artifact descriptor: cryptographic signature over canonical JSON.
pub(crate) fn admit_artifact(descriptor: &ArtifactDescriptor) -> Result<(), ArtifactError> {
    if descriptor.signature.signature_value.trim().is_empty() {
        return Err(ArtifactError::Unsigned(descriptor.artifact_id.clone()));
    }
    match descriptor.verify_canonical() {
        Ok(()) => Ok(()),
        Err(aira_object::CryptoError::MissingOrLegacy) => {
            Err(ArtifactError::Unsigned(descriptor.artifact_id.clone()))
        }
        Err(_) => Err(ArtifactError::InvalidSignature(
            descriptor.artifact_id.clone(),
        )),
    }
}

/// Verify-on-read: re-check canonical signature before returning a stored descriptor.
pub(crate) fn verify_stored_artifact(
    descriptor: ArtifactDescriptor,
) -> Result<ArtifactDescriptor, ArtifactError> {
    admit_artifact(&descriptor)?;
    Ok(descriptor)
}

/// Result of publishing an artifact.
#[derive(Debug, Clone)]
pub struct PublishResult {
    pub descriptor: ArtifactDescriptor,
    pub cas_path: PathBuf,
}

/// Supersession record (old unchanged; new artifact created).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupersessionMeta {
    pub previous: AiraRef,
    pub current: AiraRef,
}

/// Artifact store API.
pub trait ArtifactStore {
    fn publish(
        &mut self,
        descriptor: ArtifactDescriptor,
        payload: &[u8],
    ) -> Result<PublishResult, ArtifactError>;

    fn resolve(
        &self,
        artifact_id: &AiraRef,
    ) -> Result<(ArtifactDescriptor, Vec<u8>), ArtifactError>;

    fn replace_payload(
        &mut self,
        artifact_id: &AiraRef,
        _payload: &[u8],
    ) -> Result<(), ArtifactError> {
        Err(ArtifactError::Immutable(artifact_id.clone()))
    }

    fn supersede(
        &mut self,
        previous: &AiraRef,
        new_descriptor: ArtifactDescriptor,
        payload: &[u8],
    ) -> Result<SupersessionMeta, ArtifactError>;
}

/// Filesystem CAS artifact store.
pub struct CasArtifactStore {
    root: PathBuf,
    /// artifact_id → descriptor
    index: HashMap<String, ArtifactDescriptor>,
    /// previous_id → current_id
    supersessions: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Default)]
struct IndexFile {
    artifacts: HashMap<String, ArtifactDescriptor>,
    supersessions: HashMap<String, String>,
}

impl CasArtifactStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ArtifactError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(root.join("sha256"))
            .map_err(|e| ArtifactError::Storage(e.to_string()))?;
        let mut store = Self {
            root,
            index: HashMap::new(),
            supersessions: HashMap::new(),
        };
        store.load_index()?;
        Ok(store)
    }

    fn index_path(&self) -> PathBuf {
        self.root.join("index.json")
    }

    /// Persist artifact id → descriptor index for reopen across processes.
    pub fn save_index(&self) -> Result<(), ArtifactError> {
        let file = IndexFile {
            artifacts: self.index.clone(),
            supersessions: self.supersessions.clone(),
        };
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| ArtifactError::Storage(e.to_string()))?;
        fs::write(self.index_path(), json).map_err(|e| ArtifactError::Storage(e.to_string()))?;
        Ok(())
    }

    fn load_index(&mut self) -> Result<(), ArtifactError> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(());
        }
        let raw = fs::read_to_string(&path).map_err(|e| ArtifactError::Storage(e.to_string()))?;
        let file: IndexFile =
            serde_json::from_str(&raw).map_err(|e| ArtifactError::Storage(e.to_string()))?;
        self.index = file.artifacts;
        self.supersessions = file.supersessions;
        Ok(())
    }

    fn cas_path_for(root: &Path, hash: &ContentHash) -> Result<PathBuf, ArtifactError> {
        let s = hash.as_str();
        let hex = s
            .strip_prefix("sha256:")
            .ok_or_else(|| ArtifactError::Storage(format!("unsupported hash {s}")))?;
        if hex.len() < 4 {
            return Err(ArtifactError::Storage("hash too short".into()));
        }
        Ok(root
            .join("sha256")
            .join(&hex[0..2])
            .join(&hex[2..4])
            .join(format!("{hex}.bin")))
    }
}

impl ArtifactStore for CasArtifactStore {
    fn publish(
        &mut self,
        mut descriptor: ArtifactDescriptor,
        payload: &[u8],
    ) -> Result<PublishResult, ArtifactError> {
        admit_artifact(&descriptor)?;
        let actual = ContentHash::sha256_bytes(payload);
        if actual != descriptor.content_hash {
            return Err(ArtifactError::HashMismatch {
                expected: descriptor.content_hash.as_str().to_string(),
                actual: actual.as_str().to_string(),
            });
        }
        let key = descriptor.artifact_id.as_str().to_string();
        if self.index.contains_key(&key) {
            return Err(ArtifactError::Immutable(descriptor.artifact_id));
        }

        let cas_path = Self::cas_path_for(&self.root, &descriptor.content_hash)?;
        if let Some(parent) = cas_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ArtifactError::Storage(e.to_string()))?;
        }
        if !cas_path.exists() {
            fs::write(&cas_path, payload).map_err(|e| ArtifactError::Storage(e.to_string()))?;
        }
        descriptor.content_ref = format!("cas://{}", descriptor.content_hash.as_str());

        let meta_path = cas_path.with_extension("json");
        // Only write descriptor sidecar if absent (same CAS content may be shared).
        if !meta_path.exists() {
            let json = serde_json::to_string_pretty(&descriptor)
                .map_err(|e| ArtifactError::Storage(e.to_string()))?;
            fs::write(&meta_path, json).map_err(|e| ArtifactError::Storage(e.to_string()))?;
        }

        self.index.insert(key, descriptor.clone());
        self.save_index()?;
        Ok(PublishResult {
            descriptor,
            cas_path,
        })
    }

    fn resolve(
        &self,
        artifact_id: &AiraRef,
    ) -> Result<(ArtifactDescriptor, Vec<u8>), ArtifactError> {
        self.resolve_with_access(artifact_id, false)
    }

    fn supersede(
        &mut self,
        previous: &AiraRef,
        new_descriptor: ArtifactDescriptor,
        payload: &[u8],
    ) -> Result<SupersessionMeta, ArtifactError> {
        if !self.index.contains_key(previous.as_str()) {
            return Err(ArtifactError::NotFound(previous.clone()));
        }
        let published = self.publish(new_descriptor, payload)?;
        self.supersessions.insert(
            previous.as_str().to_string(),
            published.descriptor.artifact_id.as_str().to_string(),
        );
        Ok(SupersessionMeta {
            previous: previous.clone(),
            current: published.descriptor.artifact_id,
        })
    }
}

impl CasArtifactStore {
    /// Resolve with explicit private-access grant (default deny for private artifacts).
    pub fn resolve_with_access(
        &self,
        artifact_id: &AiraRef,
        allow_private: bool,
    ) -> Result<(ArtifactDescriptor, Vec<u8>), ArtifactError> {
        let desc = self
            .index
            .get(artifact_id.as_str())
            .cloned()
            .ok_or_else(|| ArtifactError::NotFound(artifact_id.clone()))?;
        let desc = verify_stored_artifact(desc)?;
        if is_private_artifact(&desc) && !allow_private {
            return Err(ArtifactError::AccessDenied(artifact_id.clone()));
        }
        let path = Self::cas_path_for(&self.root, &desc.content_hash)?;
        let bytes = fs::read(&path).map_err(|e| ArtifactError::Storage(e.to_string()))?;
        let actual = ContentHash::sha256_bytes(&bytes);
        if actual != desc.content_hash {
            return Err(ArtifactError::HashMismatch {
                expected: desc.content_hash.as_str().to_string(),
                actual: actual.as_str().to_string(),
            });
        }
        let meta_path = path.with_extension("json");
        if meta_path.exists() {
            let raw = fs::read_to_string(&meta_path)
                .map_err(|e| ArtifactError::Storage(e.to_string()))?;
            let sidecar: ArtifactDescriptor =
                serde_json::from_str(&raw).map_err(|e| ArtifactError::Storage(e.to_string()))?;
            let sidecar = verify_stored_artifact(sidecar)?;
            if sidecar.content_hash != desc.content_hash {
                return Err(ArtifactError::HashMismatch {
                    expected: desc.content_hash.as_str().to_string(),
                    actual: sidecar.content_hash.as_str().to_string(),
                });
            }
        }
        Ok((desc, bytes))
    }
}
