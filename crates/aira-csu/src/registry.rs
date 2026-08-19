//! Local CSU Registry (Issue #36).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use aira_event::{EventDescriptor, EventSink, EventType};
use aira_object::{AiraRef, ContentHash, Signature, Timestamp};
use serde::{Deserialize, Serialize};

use crate::error::CsuError;
use crate::lifecycle::CsuLifecycleState;
use crate::manifest::CsuManifest;

/// Registered CSU entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisteredCsu {
    pub manifest: CsuManifest,
    pub state: CsuLifecycleState,
}

/// Local in-memory (+ optional file) CSU registry.
#[derive(Debug, Default)]
pub struct CsuRegistry {
    entries: HashMap<String, RegisteredCsu>,
    event_seq: u64,
    producer: Option<AiraRef>,
    signer: Option<Signature>,
}

impl CsuRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure identity used for lifecycle transition events.
    pub fn with_event_identity(mut self, producer: AiraRef, signer: Signature) -> Self {
        self.producer = Some(producer);
        self.signer = Some(signer);
        self
    }

    pub fn list(&self) -> Vec<&RegisteredCsu> {
        let mut v: Vec<_> = self.entries.values().collect();
        v.sort_by(|a, b| a.manifest.csu_id.as_str().cmp(b.manifest.csu_id.as_str()));
        v
    }

    pub fn get(&self, csu_id: &AiraRef) -> Option<&RegisteredCsu> {
        self.entries.get(csu_id.as_str())
    }

    pub fn get_mut(&mut self, csu_id: &AiraRef) -> Option<&mut RegisteredCsu> {
        self.entries.get_mut(csu_id.as_str())
    }

    /// Register a manifest (→ Registered). Checks ABI + signature presence.
    pub fn register(
        &mut self,
        manifest: CsuManifest,
        events: Option<&mut dyn EventSink>,
    ) -> Result<&RegisteredCsu, CsuError> {
        manifest.validate_for_registration()?;
        let id = manifest.csu_id.as_str().to_string();
        if self.entries.contains_key(&id) {
            return Err(CsuError::ManifestInvalid(format!(
                "already registered: {id}"
            )));
        }
        let entry = RegisteredCsu {
            manifest,
            state: CsuLifecycleState::Registered,
        };
        self.entries.insert(id.clone(), entry);
        let csu_ref = AiraRef::parse(&id).map_err(|e| CsuError::Storage(e.to_string()))?;
        self.emit_lifecycle(events, &csu_ref, EventType::CSURegistered)?;
        Ok(self.entries.get(&id).expect("just inserted"))
    }

    /// Apply lifecycle transition with validation + optional event.
    pub fn transition(
        &mut self,
        csu_id: &AiraRef,
        to: CsuLifecycleState,
        events: Option<&mut dyn EventSink>,
    ) -> Result<&RegisteredCsu, CsuError> {
        let entry = self
            .entries
            .get_mut(csu_id.as_str())
            .ok_or_else(|| CsuError::NotFound(csu_id.clone()))?;
        let next = entry.state.transition(to)?;
        entry.state = next;
        let emit_type = match next {
            CsuLifecycleState::Registered => EventType::CSURegistered,
            CsuLifecycleState::Suspended => EventType::CSUSuspended,
            _ => EventType::CustomEvent,
        };
        self.emit_lifecycle(events, csu_id, emit_type)?;
        Ok(self.entries.get(csu_id.as_str()).expect("exists"))
    }

    /// Convenience: Registered → Verified → Active.
    pub fn activate(
        &mut self,
        csu_id: &AiraRef,
        events: Option<&mut dyn EventSink>,
    ) -> Result<&RegisteredCsu, CsuError> {
        self.transition(csu_id, CsuLifecycleState::Verified, None)?;
        self.transition(csu_id, CsuLifecycleState::Active, events)
    }

    pub fn suspend(
        &mut self,
        csu_id: &AiraRef,
        events: Option<&mut dyn EventSink>,
    ) -> Result<&RegisteredCsu, CsuError> {
        self.transition(csu_id, CsuLifecycleState::Suspended, events)
    }

    /// Persist registry as JSON array of entries.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), CsuError> {
        let list: Vec<&RegisteredCsu> = self.list();
        let json =
            serde_json::to_string_pretty(&list).map_err(|e| CsuError::Storage(e.to_string()))?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).map_err(|e| CsuError::Storage(e.to_string()))?;
        }
        fs::write(path, json).map_err(|e| CsuError::Storage(e.to_string()))
    }

    /// Load registry from JSON array.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CsuError> {
        let text = fs::read_to_string(path).map_err(|e| CsuError::Storage(e.to_string()))?;
        let list: Vec<RegisteredCsu> =
            serde_json::from_str(&text).map_err(|e| CsuError::Storage(e.to_string()))?;
        let mut reg = Self::new();
        for entry in list {
            entry.manifest.validate_for_registration()?;
            reg.entries
                .insert(entry.manifest.csu_id.as_str().to_string(), entry);
        }
        Ok(reg)
    }

    /// Emit a lifecycle event signed as the CSU's `publisher_identity` when registered;
    /// otherwise fall back to `with_event_identity` (primary). Fail closed on missing key.
    fn emit_lifecycle(
        &mut self,
        events: Option<&mut dyn EventSink>,
        subject: &AiraRef,
        event_type: EventType,
    ) -> Result<(), CsuError> {
        let Some(log) = events else {
            return Ok(());
        };
        self.event_seq += 1;
        let id = format!("aira:event:csulife{}", self.event_seq);
        let ev = if let Some(entry) = self.entries.get(subject.as_str()) {
            crate::support::make_event_as(
                subject.clone(),
                entry.manifest.publisher_identity.clone(),
                &id,
                event_type,
                vec![subject.clone()],
                vec![],
                vec![],
                None,
            )
            .map_err(|e| CsuError::Dispatch(e.to_string()))?
        } else {
            let (Some(producer), Some(signer)) = (self.producer.clone(), self.signer.clone())
            else {
                return Ok(());
            };
            EventDescriptor {
                event_id: AiraRef::parse(&id).map_err(|e| CsuError::Storage(e.to_string()))?,
                event_type,
                schema_version: "0.1".into(),
                producer_identity: producer,
                causal_refs: vec![],
                object_refs: vec![subject.clone()],
                artifact_refs: vec![],
                policy_refs: vec![],
                payload_hash: ContentHash::parse(
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                )
                .map_err(|e| CsuError::Storage(e.to_string()))?,
                payload_ref: None,
                created_at: Timestamp::parse("2026-07-10T12:00:00Z")
                    .map_err(|e| CsuError::Storage(e.to_string()))?,
                signature: signer,
            }
            .attach_canonical_signature()
            .map_err(|e| CsuError::Dispatch(e.to_string()))?
        };
        log.append(ev).map_err(|e| CsuError::Storage(e.to_string()))
    }
}
