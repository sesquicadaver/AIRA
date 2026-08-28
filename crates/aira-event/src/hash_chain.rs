//! Append-only event hash-chain tip (QUEUE #154).
//!
//! Links successive `EventDescriptor` entries with `prev_hash` / `entry_hash`.
//! Does not replace `MemoryEventLog` and does not wire `LocalSession` (#157).

use aira_object::ContentHash;
use thiserror::Error;

use crate::descriptor::EventDescriptor;

/// Domain-separated genesis tip for an empty chain.
pub const EVENT_LOG_CHAIN_GENESIS: &str = "aira:event-log:v0:genesis";

/// Errors from hash-chain verify / append.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChainError {
    #[error("broken hash-chain link at index {index}")]
    BrokenLink { index: usize },
    #[error("entry hash mismatch at index {index}")]
    EntryHashMismatch { index: usize },
    #[error("stored tip does not match recomputed tip")]
    TipMismatch,
    #[error("event missing or invalid canonical content hash")]
    InvalidEventHash,
}

/// One append-only chain record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainedEvent {
    pub prev_hash: ContentHash,
    pub event: EventDescriptor,
    pub entry_hash: ContentHash,
}

/// In-memory hash-chained event sequence with an explicit tip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHashChain {
    records: Vec<ChainedEvent>,
    tip: ContentHash,
}

impl Default for EventHashChain {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHashChain {
    /// Empty chain tipped at genesis.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            tip: genesis_tip(),
        }
    }

    /// Genesis tip hash (`sha256` of [`EVENT_LOG_CHAIN_GENESIS`]).
    pub fn tip(&self) -> &ContentHash {
        &self.tip
    }

    pub fn records(&self) -> &[ChainedEvent] {
        &self.records
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Append an event: `entry_hash = H(prev_tip || event.canonical_content_hash)`.
    pub fn append(&mut self, event: EventDescriptor) -> Result<&ChainedEvent, ChainError> {
        let event_hash = event
            .canonical_content_hash()
            .map_err(|_| ChainError::InvalidEventHash)?;
        let prev_hash = self.tip.clone();
        let entry_hash = link_hash(&prev_hash, &event_hash);
        self.records.push(ChainedEvent {
            prev_hash,
            event,
            entry_hash: entry_hash.clone(),
        });
        self.tip = entry_hash;
        Ok(self.records.last().expect("just pushed"))
    }

    /// Recompute the chain from genesis; fail on mid-log tamper or tip drift.
    pub fn verify_tip(&self) -> Result<(), ChainError> {
        let mut tip = genesis_tip();
        for (index, rec) in self.records.iter().enumerate() {
            if rec.prev_hash != tip {
                return Err(ChainError::BrokenLink { index });
            }
            let event_hash = rec
                .event
                .canonical_content_hash()
                .map_err(|_| ChainError::InvalidEventHash)?;
            let expected = link_hash(&rec.prev_hash, &event_hash);
            if rec.entry_hash != expected {
                return Err(ChainError::EntryHashMismatch { index });
            }
            tip = rec.entry_hash.clone();
        }
        if tip != self.tip {
            return Err(ChainError::TipMismatch);
        }
        Ok(())
    }
}

/// Genesis tip content hash.
pub fn genesis_tip() -> ContentHash {
    ContentHash::sha256_bytes(EVENT_LOG_CHAIN_GENESIS.as_bytes())
}

/// `H(prev || "|" || event_hash)` over UTF-8 hash strings.
pub fn link_hash(prev: &ContentHash, event_hash: &ContentHash) -> ContentHash {
    let mut buf = String::with_capacity(prev.as_str().len() + event_hash.as_str().len() + 1);
    buf.push_str(prev.as_str());
    buf.push('|');
    buf.push_str(event_hash.as_str());
    ContentHash::sha256_bytes(buf.as_bytes())
}

#[cfg(test)]
fn sample_event_for_chain(event_id: &str, payload_ref: Option<&str>) -> EventDescriptor {
    use aira_object::{AiraRef, Timestamp};

    use crate::descriptor::EventType;

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

#[cfg(test)]
#[test]
fn event_log_hash_chain_tip_append_verify_and_mid_tamper_detect() {
    let mut chain = EventHashChain::new();
    assert_eq!(chain.tip(), &genesis_tip());
    chain.verify_tip().unwrap();

    chain
        .append(sample_event_for_chain("aira:event:hc1", Some("a")))
        .unwrap();
    chain
        .append(sample_event_for_chain("aira:event:hc2", Some("b")))
        .unwrap();
    assert_eq!(chain.len(), 2);
    chain.verify_tip().unwrap();
    let tip_after = chain.tip().clone();

    // Mid-log tamper: mutate stored event payload without updating entry_hash.
    chain.records[0].event.payload_ref = Some("tampered".into());
    let err = chain.verify_tip().unwrap_err();
    assert!(matches!(
        err,
        ChainError::EntryHashMismatch { index: 0 } | ChainError::BrokenLink { index: 1 }
    ));

    // Restore event but break entry_hash directly.
    chain.records[0].event = sample_event_for_chain("aira:event:hc1", Some("a"));
    chain.records[0].entry_hash = ContentHash::sha256_bytes(b"bogus-entry");
    assert_eq!(
        chain.verify_tip().unwrap_err(),
        ChainError::EntryHashMismatch { index: 0 }
    );

    // Tip drift with intact records.
    let mut ok = EventHashChain::new();
    ok.append(sample_event_for_chain("aira:event:hc3", Some("c")))
        .unwrap();
    ok.verify_tip().unwrap();
    ok.tip = tip_after;
    assert_eq!(ok.verify_tip().unwrap_err(), ChainError::TipMismatch);
}
