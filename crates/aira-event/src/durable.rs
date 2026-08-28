//! Durable file-backed hash-chain event log (QUEUE #156).
//!
//! Persists [`EventHashChain`] as JSON. Wired from `LocalSession` / `init_node` in
//! `aira-flow` (QUEUE #157).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::descriptor::EventDescriptor;
use crate::hash_chain::{ChainError, ChainedEvent, EventHashChain};

/// On-disk schema id for the file-chain event log.
pub const FILE_CHAIN_EVENT_LOG_SCHEMA: &str = "aira:event-log:file-chain:0.1";

/// Errors from durable file-chain event log I/O.
#[derive(Debug, Error)]
pub enum DurableEventError {
    #[error("io: {0}")]
    Io(String),
    #[error("json: {0}")]
    Json(String),
    #[error(transparent)]
    Chain(#[from] ChainError),
}

/// JSON envelope written to disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct FileChainEventLogFile {
    schema: String,
    records: Vec<ChainedEvent>,
}

/// Append-only file-backed event log using the in-memory hash chain.
#[derive(Debug)]
pub struct FileChainEventLog {
    path: PathBuf,
    chain: EventHashChain,
}

impl FileChainEventLog {
    /// Open an existing log or create an empty one at `path`.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, DurableEventError> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            Self::open(path)
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| DurableEventError::Io(e.to_string()))?;
            }
            let log = Self {
                path,
                chain: EventHashChain::new(),
            };
            log.persist()?;
            Ok(log)
        }
    }

    /// Open and verify an existing file-chain log.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DurableEventError> {
        let path = path.as_ref().to_path_buf();
        let raw = fs::read_to_string(&path).map_err(|e| DurableEventError::Io(e.to_string()))?;
        let file: FileChainEventLogFile =
            serde_json::from_str(&raw).map_err(|e| DurableEventError::Json(e.to_string()))?;
        if file.schema != FILE_CHAIN_EVENT_LOG_SCHEMA {
            return Err(DurableEventError::Json(format!(
                "unsupported schema: {}",
                file.schema
            )));
        }
        let chain = EventHashChain::try_from_records(file.records)?;
        Ok(Self { path, chain })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn chain(&self) -> &EventHashChain {
        &self.chain
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    /// Append one event and flush the full chain to disk.
    pub fn append(&mut self, event: EventDescriptor) -> Result<&ChainedEvent, DurableEventError> {
        self.chain.append(event)?;
        self.persist()?;
        Ok(self
            .chain
            .records()
            .last()
            .expect("append leaves at least one record"))
    }

    fn persist(&self) -> Result<(), DurableEventError> {
        let file = FileChainEventLogFile {
            schema: FILE_CHAIN_EVENT_LOG_SCHEMA.into(),
            records: self.chain.records().to_vec(),
        };
        let body = serde_json::to_string_pretty(&file)
            .map_err(|e| DurableEventError::Json(e.to_string()))?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, body.as_bytes()).map_err(|e| DurableEventError::Io(e.to_string()))?;
        fs::rename(&tmp, &self.path).map_err(|e| DurableEventError::Io(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{AiraRef, ContentHash, Timestamp};

    use crate::descriptor::EventType;

    fn sample_event(event_id: &str, payload_ref: Option<&str>) -> EventDescriptor {
        let payload_hash = ContentHash::parse(
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .unwrap();
        let e = EventDescriptor {
            event_id: AiraRef::parse(event_id).unwrap(),
            event_type: EventType::ProblemSubmitted,
            schema_version: "0.1".into(),
            producer_identity: AiraRef::parse("aira:identity:local-test").unwrap(),
            causal_refs: vec![],
            object_refs: vec![AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()],
            artifact_refs: vec![AiraRef::parse(
                "aira:artifact:sha256_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            )
            .unwrap()],
            policy_refs: vec![],
            payload_hash: payload_hash.clone(),
            payload_ref: payload_ref.map(str::to_string),
            created_at: Timestamp::parse("2026-07-10T12:00:00Z").unwrap(),
            signature: aira_object::local_test_signature(payload_hash.as_str().as_bytes()),
        };
        e.attach_canonical_signature().expect("canonical sample")
    }

    #[test]
    fn file_chain_event_log_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events").join("file-chain-log.json");

        let mut log = FileChainEventLog::open_or_create(&path).unwrap();
        assert!(path.exists());
        assert!(log.is_empty());
        log.append(sample_event("aira:event:d1", Some("one")))
            .unwrap();
        log.append(sample_event("aira:event:d2", Some("two")))
            .unwrap();
        assert_eq!(log.len(), 2);
        let tip = log.chain().tip().clone();
        drop(log);

        let reopened = FileChainEventLog::open(&path).unwrap();
        assert_eq!(reopened.len(), 2);
        assert_eq!(reopened.chain().tip(), &tip);
        reopened.chain().verify_tip().unwrap();
        assert_eq!(
            reopened.chain().records()[0].event.event_id.as_str(),
            "aira:event:d1"
        );
        assert_eq!(
            reopened.chain().records()[1].event.payload_ref.as_deref(),
            Some("two")
        );
    }

    #[test]
    fn file_chain_event_log_rejects_tampered_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tamper.json");
        let mut log = FileChainEventLog::open_or_create(&path).unwrap();
        log.append(sample_event("aira:event:t1", Some("ok")))
            .unwrap();
        drop(log);

        let mut raw = fs::read_to_string(&path).unwrap();
        raw = raw.replace("\"ok\"", "\"tampered\"");
        fs::write(&path, raw).unwrap();

        let err = FileChainEventLog::open(&path).unwrap_err();
        assert!(matches!(
            err,
            DurableEventError::Chain(ChainError::EntryHashMismatch { .. })
        ));
    }
}
