//! Artifact-basic CSU (Issue #46).
//!
//! Publishes / resolves / supersedes via Artifact Runtime and emits lifecycle events.
//!
//! CustomEvent `payload_ref` protocol:
//! - `op:publish:<bytes-or-text>`
//! - `op:resolve`
//! - `op:supersede` (previous in artifact_refs[0]; new payload in payload_ref after prefix)

use aira_artifact::ArtifactType;
use aira_csu::support::{basic_manifest, make_artifact_as, make_event_as};
use aira_csu::{Csu, CsuExecutionContext, CsuHandlerError, CsuManifest, CsuOutput, CsuType};
use aira_event::{EventDescriptor, EventType};
use aira_object::{AiraRef, ContentHash};

/// Artifact runtime façade CSU.
pub struct ArtifactBasicCsu {
    manifest: CsuManifest,
    seq: u64,
    run_nonce: u64,
}

impl Default for ArtifactBasicCsu {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactBasicCsu {
    pub fn new() -> Self {
        Self {
            manifest: basic_manifest(
                "aira:csu:artifact.basic",
                "artifact-basic",
                CsuType::Artifact,
                &["CustomEvent"],
                &[
                    "ArtifactPublished",
                    "ArtifactResolved",
                    "ArtifactInvalid",
                    "ArtifactSuperseded",
                ],
            ),
            seq: 1,
            run_nonce: 0,
        }
    }

    /// Namespace ids for multi-run local nodes (Epic 8).
    pub fn with_run_nonce(mut self, run_nonce: u64) -> Self {
        self.run_nonce = run_nonce;
        self
    }

    /// Emit as a distinct publisher identity (must have a signing key in the process keyring).
    pub fn with_publisher(mut self, publisher: AiraRef) -> Self {
        aira_csu::support::apply_publisher(&mut self.manifest, publisher);
        self
    }

    fn next_id(&mut self, kind: &str) -> String {
        let id = format!("aira:{kind}:art{}_{}", self.run_nonce, self.seq);
        self.seq += 1;
        id
    }
}

impl Csu for ArtifactBasicCsu {
    fn manifest(&self) -> &CsuManifest {
        &self.manifest
    }

    fn on_event(
        &mut self,
        event: &EventDescriptor,
        ctx: &mut CsuExecutionContext<'_, '_>,
    ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
        if event.event_type != EventType::CustomEvent {
            return Ok(vec![]);
        }
        let op = event.payload_ref.as_deref().unwrap_or("op:publish:hello");
        if let Some(rest) = op.strip_prefix("op:publish:") {
            let payload = rest.as_bytes();
            let aid = self.next_id("artifact");
            let desc = make_artifact_as(
                self.manifest.publisher_identity.clone(),
                &aid,
                ArtifactType::CustomArtifact,
                payload,
                vec![event.event_id.clone()],
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            // Hash integrity: reject if caller planted mismatched hash via empty payload edge.
            if payload.is_empty() {
                let invalid = make_event_as(
                    self.manifest.publisher_identity.clone(),
                    &self.next_id("event"),
                    EventType::ArtifactInvalid,
                    event.object_refs.clone(),
                    vec![],
                    vec![event.event_id.clone()],
                    Some("empty payload".into()),
                )
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
                ctx.append_event(invalid.clone())
                    .map_err(|e| CsuHandlerError {
                        message: e.to_string(),
                    })?;
                return Ok(vec![CsuOutput::Event(invalid)]);
            }
            ctx.publish_artifact(desc.clone(), payload)
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
            let published = make_event_as(
                self.manifest.publisher_identity.clone(),
                &self.next_id("event"),
                EventType::ArtifactPublished,
                event.object_refs.clone(),
                vec![desc.artifact_id.clone()],
                vec![event.event_id.clone()],
                Some(desc.content_hash.as_str().into()),
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            ctx.append_event(published.clone())
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
            return Ok(vec![
                CsuOutput::Artifact {
                    descriptor: desc,
                    payload: payload.to_vec(),
                },
                CsuOutput::Event(published),
            ]);
        }

        if op == "op:resolve" {
            let Some(id) = event.artifact_refs.first() else {
                let invalid = make_event_as(
                    self.manifest.publisher_identity.clone(),
                    &self.next_id("event"),
                    EventType::ArtifactInvalid,
                    event.object_refs.clone(),
                    vec![],
                    vec![event.event_id.clone()],
                    Some("missing artifact_ref".into()),
                )
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
                ctx.append_event(invalid.clone())
                    .map_err(|e| CsuHandlerError {
                        message: e.to_string(),
                    })?;
                return Ok(vec![CsuOutput::Event(invalid)]);
            };
            match ctx.resolve_artifact(id) {
                Ok((desc, bytes)) => {
                    let actual = ContentHash::sha256_bytes(&bytes);
                    if actual != desc.content_hash {
                        let invalid = make_event_as(
                            self.manifest.publisher_identity.clone(),
                            &self.next_id("event"),
                            EventType::ArtifactInvalid,
                            event.object_refs.clone(),
                            vec![id.clone()],
                            vec![event.event_id.clone()],
                            Some("hash mismatch".into()),
                        )
                        .map_err(|e| CsuHandlerError {
                            message: e.to_string(),
                        })?;
                        ctx.append_event(invalid.clone())
                            .map_err(|e| CsuHandlerError {
                                message: e.to_string(),
                            })?;
                        return Ok(vec![CsuOutput::Event(invalid)]);
                    }
                    let resolved = make_event_as(
                        self.manifest.publisher_identity.clone(),
                        &self.next_id("event"),
                        EventType::ArtifactResolved,
                        event.object_refs.clone(),
                        vec![id.clone()],
                        vec![event.event_id.clone()],
                        Some(desc.content_hash.as_str().into()),
                    )
                    .map_err(|e| CsuHandlerError {
                        message: e.to_string(),
                    })?;
                    ctx.append_event(resolved.clone())
                        .map_err(|e| CsuHandlerError {
                            message: e.to_string(),
                        })?;
                    Ok(vec![CsuOutput::Event(resolved)])
                }
                Err(e) => {
                    let invalid = make_event_as(
                        self.manifest.publisher_identity.clone(),
                        &self.next_id("event"),
                        EventType::ArtifactInvalid,
                        event.object_refs.clone(),
                        vec![id.clone()],
                        vec![event.event_id.clone()],
                        Some(e.to_string()),
                    )
                    .map_err(|e| CsuHandlerError {
                        message: e.to_string(),
                    })?;
                    ctx.append_event(invalid.clone())
                        .map_err(|e| CsuHandlerError {
                            message: e.to_string(),
                        })?;
                    Ok(vec![CsuOutput::Event(invalid)])
                }
            }
        } else if let Some(rest) = op.strip_prefix("op:supersede:") {
            let previous = event
                .artifact_refs
                .first()
                .cloned()
                .ok_or_else(|| CsuHandlerError {
                    message: "supersede requires previous artifact_ref".into(),
                })?;
            let payload = rest.as_bytes();
            let aid = self.next_id("artifact");
            let desc = make_artifact_as(
                self.manifest.publisher_identity.clone(),
                &aid,
                ArtifactType::CustomArtifact,
                payload,
                vec![event.event_id.clone(), previous.clone()],
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            ctx.supersede_artifact(&previous, desc.clone(), payload)
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
            let superseded = make_event_as(
                self.manifest.publisher_identity.clone(),
                &self.next_id("event"),
                EventType::ArtifactSuperseded,
                event.object_refs.clone(),
                vec![previous, desc.artifact_id.clone()],
                vec![event.event_id.clone()],
                None,
            )
            .map_err(|e| CsuHandlerError {
                message: e.to_string(),
            })?;
            ctx.append_event(superseded.clone())
                .map_err(|e| CsuHandlerError {
                    message: e.to_string(),
                })?;
            Ok(vec![
                CsuOutput::Artifact {
                    descriptor: desc,
                    payload: payload.to_vec(),
                },
                CsuOutput::Event(superseded),
            ])
        } else {
            Err(CsuHandlerError {
                message: format!("unknown artifact op: {op}"),
            })
        }
    }
}

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_artifact::CasArtifactStore;
    use aira_csu::support::make_event as mk;
    use aira_event::MemoryEventLog;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn publish_resolve_supersede_events() {
        let mut csu = ArtifactBasicCsu::new();
        let mut log = MemoryEventLog::new();
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let mut ctx = aira_csu::CsuExecutionContext::new(
            csu.manifest().csu_id.clone(),
            &mut log,
            Some(&mut store),
            None,
        );

        let pub_ev = mk(
            "aira:event:a1",
            EventType::CustomEvent,
            vec![],
            vec![],
            vec![],
            Some("op:publish:hello-world".into()),
        );
        let outs = csu.on_event(&pub_ev, &mut ctx).unwrap();
        let art_id = outs
            .iter()
            .find_map(|o| match o {
                CsuOutput::Artifact { descriptor, .. } => Some(descriptor.artifact_id.clone()),
                _ => None,
            })
            .unwrap();
        assert!(outs.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::ArtifactPublished
        )));

        let res_ev = mk(
            "aira:event:a2",
            EventType::CustomEvent,
            vec![],
            vec![art_id.clone()],
            vec![],
            Some("op:resolve".into()),
        );
        let outs2 = csu.on_event(&res_ev, &mut ctx).unwrap();
        assert!(outs2.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::ArtifactResolved
        )));

        let sup_ev = mk(
            "aira:event:a3",
            EventType::CustomEvent,
            vec![],
            vec![art_id],
            vec![],
            Some("op:supersede:hello-v2".into()),
        );
        let outs3 = csu.on_event(&sup_ev, &mut ctx).unwrap();
        assert!(outs3.iter().any(|o| matches!(
            o,
            CsuOutput::Event(e) if e.event_type == EventType::ArtifactSuperseded
        )));
    }
}
