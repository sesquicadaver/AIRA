//! Core Object Descriptor (Schema Pack §4).

use serde::{Deserialize, Serialize};

use crate::types::{AiraRef, ContentHash, Signature, Timestamp};

/// Canonical Core object types (Book 0 ontology). Forbidden: GPU/Node/Driver/…
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    ProblemStatement,
    Context,
    Evidence,
    EpistemicStatus,
    ExecutionIntent,
    ExecutionCapsule,
    Capability,
    Artifact,
    Event,
    Policy,
    #[serde(rename = "CSU")]
    Csu,
    VerifiedResultArtifact,
}

/// Core Object Descriptor — immutable metadata for a stored object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectDescriptor {
    pub object_id: AiraRef,
    pub object_type: ObjectType,
    pub schema_version: String,
    pub created_at: Timestamp,
    pub producer_identity: AiraRef,
    pub policy_refs: Vec<AiraRef>,
    pub provenance_refs: Vec<AiraRef>,
    pub content_hash: ContentHash,
    pub signature: Signature,
}

impl ObjectDescriptor {
    /// Fixture-friendly ProblemStatement descriptor for tests.
    pub fn example_problem() -> Self {
        Self {
            object_id: AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap(),
            object_type: ObjectType::ProblemStatement,
            schema_version: "0.1".into(),
            created_at: Timestamp::parse("2026-07-10T12:00:00Z").unwrap(),
            producer_identity: AiraRef::parse("aira:identity:local-test").unwrap(),
            policy_refs: vec![AiraRef::parse("aira:policy:default").unwrap()],
            provenance_refs: vec![],
            content_hash: ContentHash::parse(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            )
            .unwrap(),
            signature: crate::local_test_signature(
                b"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
        }
    }
}
