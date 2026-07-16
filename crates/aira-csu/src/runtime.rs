//! In-process CSU trait, execution context, dispatch, isolation (Issues #38–#40).

use std::collections::HashMap;
use std::fmt;

use aira_artifact::{ArtifactDescriptor, ArtifactStore};
use aira_event::{EventDescriptor, EventSink, EventType};
use aira_object::{AiraRef, ContentHash, Signature, Timestamp};
use aira_policy::{PolicyDecision, PolicyGate, PolicyQuery};

use crate::error::CsuError;
use crate::lifecycle::CsuLifecycleState;
use crate::manifest::CsuManifest;
use crate::registry::CsuRegistry;

/// Outputs a CSU may produce (Book IV §12).
#[derive(Debug, Clone)]
pub enum CsuOutput {
    Event(EventDescriptor),
    Artifact {
        descriptor: ArtifactDescriptor,
        payload: Vec<u8>,
    },
    PolicyQuery(PolicyQuery),
    Failure {
        message: String,
    },
}

/// Errors returned from `Csu::on_event`.
#[derive(Debug)]
pub struct CsuHandlerError {
    pub message: String,
}

impl fmt::Display for CsuHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CsuHandlerError {}

/// Isolated execution context — Event / Artifact publish / Policy only.
///
/// Deliberately does **not** expose ObjectStore mutation, Artifact mutation,
/// or peer CSU invocation (Issue #40 isolation baseline).
pub struct CsuExecutionContext<'e, 'a> {
    pub csu_id: AiraRef,
    events: &'e mut dyn EventSink,
    artifacts: Option<&'a mut dyn ArtifactStore>,
    policy: Option<&'a mut PolicyGate>,
}

impl<'e, 'a> CsuExecutionContext<'e, 'a> {
    pub fn new(
        csu_id: AiraRef,
        events: &'e mut dyn EventSink,
        artifacts: Option<&'a mut dyn ArtifactStore>,
        policy: Option<&'a mut PolicyGate>,
    ) -> Self {
        Self {
            csu_id,
            events,
            artifacts,
            policy,
        }
    }

    /// Append an event via Core Event API.
    pub fn append_event(&mut self, event: EventDescriptor) -> Result<(), CsuError> {
        self.events
            .append(event)
            .map_err(|e| CsuError::Dispatch(e.to_string()))
    }

    /// Publish a new artifact (CAS). No in-place mutation API.
    pub fn publish_artifact(
        &mut self,
        descriptor: ArtifactDescriptor,
        payload: &[u8],
    ) -> Result<(), CsuError> {
        let store = self
            .artifacts
            .as_mut()
            .ok_or_else(|| CsuError::Isolation("artifact store not bound".into()))?;
        store
            .publish(descriptor, payload)
            .map_err(|e| CsuError::Dispatch(e.to_string()))?;
        Ok(())
    }

    /// Resolve an artifact by id (read-only).
    pub fn resolve_artifact(
        &self,
        artifact_id: &AiraRef,
    ) -> Result<(ArtifactDescriptor, Vec<u8>), CsuError> {
        let store = self
            .artifacts
            .as_ref()
            .ok_or_else(|| CsuError::Isolation("artifact store not bound".into()))?;
        store
            .resolve(artifact_id)
            .map_err(|e| CsuError::Dispatch(e.to_string()))
    }

    /// Supersede an artifact (old retained; new published).
    pub fn supersede_artifact(
        &mut self,
        previous: &AiraRef,
        descriptor: ArtifactDescriptor,
        payload: &[u8],
    ) -> Result<(), CsuError> {
        let store = self
            .artifacts
            .as_mut()
            .ok_or_else(|| CsuError::Isolation("artifact store not bound".into()))?;
        store
            .supersede(previous, descriptor, payload)
            .map_err(|e| CsuError::Dispatch(e.to_string()))?;
        Ok(())
    }

    /// Evaluate policy (no bypass).
    pub fn check_policy(&mut self, query: PolicyQuery) -> Result<PolicyDecision, CsuError> {
        let gate = self
            .policy
            .as_mut()
            .ok_or_else(|| CsuError::Isolation("policy gate not bound".into()))?;
        gate.check(query, Some(self.events))
            .map_err(|e| CsuError::Dispatch(e.to_string()))
    }

    /// Explicitly denied: mutate core object.
    pub fn mutate_core_object(&self, _object_id: &AiraRef) -> Result<(), CsuError> {
        Err(CsuError::Isolation(
            "CSU cannot mutate Core Object directly".into(),
        ))
    }

    /// Explicitly denied: mutate artifact in place.
    pub fn mutate_artifact(&self, _artifact_id: &AiraRef) -> Result<(), CsuError> {
        Err(CsuError::Isolation(
            "CSU cannot mutate Artifact directly".into(),
        ))
    }

    /// Explicitly denied: call another CSU.
    pub fn call_csu(&self, _peer: &AiraRef) -> Result<(), CsuError> {
        Err(CsuError::Isolation(
            "CSU cannot call another CSU directly".into(),
        ))
    }
}

/// In-process CSU contract (Issue #38).
pub trait Csu: Send {
    fn manifest(&self) -> &CsuManifest;

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError>;
}

/// Runtime that binds registry entries to in-process handlers and dispatches events.
pub struct CsuRuntime {
    pub registry: CsuRegistry,
    handlers: HashMap<String, Box<dyn Csu>>,
    producer: AiraRef,
    signer: Signature,
    fail_seq: u64,
}

impl CsuRuntime {
    pub fn new(producer: AiraRef, signer: Signature) -> Self {
        let registry = CsuRegistry::new().with_event_identity(producer.clone(), signer.clone());
        Self {
            registry,
            handlers: HashMap::new(),
            producer,
            signer,
            fail_seq: 0,
        }
    }

    /// Register manifest + bind handler instance.
    pub fn register_handler(
        &mut self,
        handler: Box<dyn Csu>,
        events: Option<&mut dyn EventSink>,
    ) -> Result<(), CsuError> {
        let manifest = handler.manifest().clone();
        let id = manifest.csu_id.as_str().to_string();
        self.registry.register(manifest, events)?;
        self.handlers.insert(id, handler);
        Ok(())
    }

    pub fn activate(
        &mut self,
        csu_id: &AiraRef,
        events: Option<&mut dyn EventSink>,
    ) -> Result<(), CsuError> {
        self.registry.activate(csu_id, events)?;
        Ok(())
    }

    pub fn suspend(
        &mut self,
        csu_id: &AiraRef,
        events: Option<&mut dyn EventSink>,
    ) -> Result<(), CsuError> {
        self.registry.suspend(csu_id, events)?;
        Ok(())
    }

    /// Replace an already-registered handler instance (same csu_id).
    pub fn replace_handler(&mut self, handler: Box<dyn Csu>) -> Result<(), CsuError> {
        let id = handler.manifest().csu_id.as_str().to_string();
        if self.registry.get(&handler.manifest().csu_id).is_none() {
            return Err(CsuError::NotFound(handler.manifest().csu_id.clone()));
        }
        self.handlers.insert(id, handler);
        Ok(())
    }

    /// Dispatch an event to all Active CSU subscribed to its type (event-only context).
    pub fn dispatch(
        &mut self,
        event: &EventDescriptor,
        events: &mut dyn EventSink,
    ) -> Result<Vec<CsuOutput>, CsuError> {
        let ids = self.active_subscribers(event);
        let mut all = Vec::new();
        for csu_id in ids {
            all.extend(self.invoke(&csu_id, event, events, None)?);
        }
        Ok(all)
    }

    /// Dispatch with ArtifactStore to all matching Active CSUs (sequential).
    pub fn dispatch_with_artifacts(
        &mut self,
        event: &EventDescriptor,
        events: &mut dyn EventSink,
        artifacts: &mut dyn ArtifactStore,
    ) -> Result<Vec<CsuOutput>, CsuError> {
        self.dispatch_all_with_artifacts(event, events, artifacts)
    }

    /// Invoke every matching Active CSU with a bound artifact store.
    pub fn dispatch_all_with_artifacts(
        &mut self,
        event: &EventDescriptor,
        events: &mut dyn EventSink,
        artifacts: &mut dyn ArtifactStore,
    ) -> Result<Vec<CsuOutput>, CsuError> {
        let ids = self.active_subscribers(event);
        self.dispatch_ids_with_artifacts(ids, event, events, artifacts)
    }

    fn dispatch_ids_with_artifacts(
        &mut self,
        mut ids: Vec<AiraRef>,
        event: &EventDescriptor,
        events: &mut dyn EventSink,
        artifacts: &mut dyn ArtifactStore,
    ) -> Result<Vec<CsuOutput>, CsuError> {
        let Some(csu_id) = ids.pop() else {
            return Ok(vec![]);
        };
        let mut outs = self.invoke(&csu_id, event, events, Some(artifacts))?;
        outs.extend(self.dispatch_ids_with_artifacts(ids, event, events, artifacts)?);
        Ok(outs)
    }

    fn active_subscribers(&self, event: &EventDescriptor) -> Vec<AiraRef> {
        let event_type_name = format!("{:?}", event.event_type);
        self.registry
            .list()
            .iter()
            .filter(|e| e.state == CsuLifecycleState::Active)
            .filter(|e| {
                e.manifest
                    .subscribed_event_types()
                    .iter()
                    .any(|t| t == &event_type_name)
            })
            .map(|e| e.manifest.csu_id.clone())
            .collect()
    }

    fn invoke(
        &mut self,
        csu_id: &AiraRef,
        event: &EventDescriptor,
        events: &mut dyn EventSink,
        artifacts: Option<&mut dyn ArtifactStore>,
    ) -> Result<Vec<CsuOutput>, CsuError> {
        let result = {
            let handler = self
                .handlers
                .get_mut(csu_id.as_str())
                .ok_or_else(|| CsuError::NotFound(csu_id.clone()))?;
            let mut ctx = CsuExecutionContext::new(csu_id.clone(), events, artifacts, None);
            handler.on_event(event, &mut ctx)
        };
        match result {
            Ok(outputs) => {
                for out in &outputs {
                    if let CsuOutput::Failure { message } = out {
                        self.emit_failed(csu_id, message, events)?;
                    }
                }
                Ok(outputs)
            }
            Err(e) => {
                self.emit_failed(csu_id, &e.message, events)?;
                Err(CsuError::Dispatch(e.message))
            }
        }
    }

    fn emit_failed(
        &mut self,
        csu_id: &AiraRef,
        message: &str,
        events: &mut dyn EventSink,
    ) -> Result<(), CsuError> {
        let _ = message;
        self.fail_seq += 1;
        let id = format!("aira:event:csufail{}", self.fail_seq);
        let ev = EventDescriptor {
            event_id: AiraRef::parse(&id).map_err(|e| CsuError::Storage(e.to_string()))?,
            event_type: EventType::CSUFailed,
            schema_version: "0.1".into(),
            producer_identity: self.producer.clone(),
            causal_refs: vec![],
            object_refs: vec![csu_id.clone()],
            artifact_refs: vec![],
            policy_refs: vec![],
            payload_hash: ContentHash::parse(
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            )
            .map_err(|e| CsuError::Storage(e.to_string()))?,
            payload_ref: None,
            created_at: Timestamp::parse("2026-07-10T12:00:00Z")
                .map_err(|e| CsuError::Storage(e.to_string()))?,
            signature: self.signer.clone(),
        };
        events
            .append(ev)
            .map_err(|e| CsuError::Storage(e.to_string()))
    }
}
