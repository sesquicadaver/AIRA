# AIRA Schema Pack v0.1

```text
Type: Normative / Schema
Scope: AIRA Data Contracts
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
  - AIRA Conformance Test Specification
Exports:
  - Core Object Descriptor Schema
  - Artifact Descriptor Schema
  - Event Descriptor Schema
  - Policy Query / Decision Schema
  - Capability Descriptor Schema
  - CSU Manifest Schema
  - Protocol Envelope Schema
  - Verified Result Artifact Schema
  - Evidence / Epistemic Schemas
  - Conformance Report Schema
```

AIRA Schema Pack v0.1 формалізує мінімальний набір структур даних, необхідний для сумісності реалізацій. Він не додає нових сутностей. Він лише перетворює Book 0–III та Conformance Specification у перевірні схеми. Book 0 фіксує канонічну онтологію і ціль `Problem Statement → Verified Result Artifact`; Book I визначає Core Object Model / ABI; Book II — protocol envelope і протокольні контракти; Book III — CSU contracts; Conformance Specification вимагає перевіряти ці схеми тестами.     

---

# 1. Purpose

Schema Pack визначає канонічні формати обміну для AIRA.

Його задача:

```text
забезпечити сумісність реалізацій;
уніфікувати Artifact / Event / Object descriptors;
зробити Conformance tests автоматизованими;
усунути неявні або неформальні структури даних;
не допустити повернення до застарілих Node / Driver / Scheduler / Blockchain-first моделей.
```

Schema Pack не описує алгоритми, runtime, storage backend, ML-моделі, routing heuristics або reference implementation.

---

# 2. Schema Format

## 2.1 Canonical Format

Основний формат:

```text
JSON Schema 2020-12
```

Допустимі похідні представлення:

```text
YAML — only for human-authored manifests/configs
CBOR — binary transport encoding
CDDL — optional compact binary schema mapping
```

## 2.2 Canonical Serialization

Для hashing/signing використовується:

```text
Canonical JSON
UTF-8
deterministic key ordering
no insignificant whitespace
SHA-256 over canonical byte representation
```

## 2.3 Schema ID Format

```text
aira:schema:<domain>:<name>:<version>
```

Приклади:

```text
aira:schema:core:object-descriptor:0.1
aira:schema:event:event-descriptor:0.1
aira:schema:artifact:artifact-descriptor:0.1
aira:schema:csu:manifest:0.1
```

---

# 3. Common Scalar Types

## 3.1 Identifier

```json
{
  "$id": "aira:schema:common:identifier:0.1",
  "type": "string",
  "pattern": "^[a-zA-Z][a-zA-Z0-9_.:-]{2,127}$"
}
```

## 3.2 AIRA Reference

```json
{
  "$id": "aira:schema:common:ref:0.1",
  "type": "string",
  "pattern": "^aira:[a-z][a-z0-9_-]*:[a-zA-Z0-9_.:-]+$"
}
```

Examples:

```text
aira:problem:01HZY...
aira:context:01HZZ...
aira:event:01J...
aira:artifact:sha256:...
aira:csu:ctx.basic
```

## 3.3 Hash

```json
{
  "$id": "aira:schema:common:hash:0.1",
  "type": "string",
  "pattern": "^(sha256|sha512):[a-fA-F0-9]+$"
}
```

## 3.4 Timestamp

```json
{
  "$id": "aira:schema:common:timestamp:0.1",
  "type": "string",
  "format": "date-time"
}
```

## 3.5 Signature

```json
{
  "$id": "aira:schema:common:signature:0.1",
  "type": "object",
  "required": ["algorithm", "key_ref", "signature_value"],
  "additionalProperties": false,
  "properties": {
    "algorithm": { "type": "string" },
    "key_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "signature_value": { "type": "string" }
  }
}
```

## 3.6 Scope Descriptor

```json
{
  "$id": "aira:schema:common:scope-descriptor:0.1",
  "type": "object",
  "additionalProperties": true,
  "required": ["scope_type"],
  "properties": {
    "scope_type": {
      "type": "string",
      "enum": ["local", "session", "user", "federation", "domain", "global", "custom"]
    },
    "description": { "type": "string" }
  }
}
```

---

# 4. Core Object Descriptor Schema

Використовується для всіх Core Objects Book I.

```json
{
  "$id": "aira:schema:core:object-descriptor:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "object_id",
    "object_type",
    "schema_version",
    "created_at",
    "producer_identity",
    "policy_refs",
    "provenance_refs",
    "content_hash",
    "signature"
  ],
  "properties": {
    "object_id": { "$ref": "aira:schema:common:ref:0.1" },
    "object_type": {
      "type": "string",
      "enum": [
        "ProblemStatement",
        "Context",
        "Evidence",
        "EpistemicStatus",
        "ExecutionIntent",
        "ExecutionCapsule",
        "Capability",
        "Artifact",
        "Event",
        "Policy",
        "CSU",
        "VerifiedResultArtifact"
      ]
    },
    "schema_version": { "type": "string" },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "producer_identity": { "$ref": "aira:schema:common:ref:0.1" },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "provenance_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "content_hash": { "$ref": "aira:schema:common:hash:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 5. Problem Statement Schema

```json
{
  "$id": "aira:schema:core:problem-statement:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "problem_id",
    "statement",
    "submitted_by",
    "created_at",
    "input_artifact_refs",
    "constraints",
    "policy_refs"
  ],
  "properties": {
    "problem_id": { "$ref": "aira:schema:common:ref:0.1" },
    "statement": { "type": "string", "minLength": 1 },
    "submitted_by": { "$ref": "aira:schema:common:ref:0.1" },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "input_artifact_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "constraints": {
      "type": "object",
      "additionalProperties": true,
      "properties": {
        "max_cost": { "type": ["number", "null"] },
        "max_latency_ms": { "type": ["integer", "null"] },
        "privacy_class": { "type": ["string", "null"] },
        "required_confidence": { "type": ["number", "null"], "minimum": 0, "maximum": 1 }
      }
    },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    }
  }
}
```

---

# 6. Context Artifact Schema

```json
{
  "$id": "aira:schema:artifact:context-artifact:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "context_id",
    "problem_statement_ref",
    "context_type",
    "resolved_factors",
    "unresolved_factors",
    "confidence",
    "scope",
    "evidence_refs",
    "provenance_refs"
  ],
  "properties": {
    "context_id": { "$ref": "aira:schema:common:ref:0.1" },
    "problem_statement_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "context_type": {
      "type": "string",
      "enum": ["session", "user", "domain", "federation", "global", "execution", "custom"]
    },
    "resolved_factors": {
      "type": "object",
      "additionalProperties": true
    },
    "unresolved_factors": {
      "type": "array",
      "items": { "type": "string" }
    },
    "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
    "scope": { "$ref": "aira:schema:common:scope-descriptor:0.1" },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "provenance_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    }
  }
}
```

---

# 7. Artifact Descriptor Schema

Artifact — основна одиниця незмінного результату, знань, доказів, політик, моделей, евристик, research outputs та conformance reports.

```json
{
  "$id": "aira:schema:artifact:artifact-descriptor:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "artifact_id",
    "artifact_type",
    "schema_version",
    "content_hash",
    "content_ref",
    "producer_identity",
    "provenance_refs",
    "dependency_refs",
    "policy_refs",
    "signature",
    "created_at"
  ],
  "properties": {
    "artifact_id": { "$ref": "aira:schema:common:ref:0.1" },
    "artifact_type": {
      "type": "string",
      "enum": [
        "VerifiedResultArtifact",
        "ReadySolutionArtifact",
        "KnowledgeArtifact",
        "EvidenceArtifact",
        "BestCurrentHypothesisArtifact",
        "NegativeResultArtifact",
        "OpenResearchArtifact",
        "OperationalArtifact",
        "ResearchArtifact",
        "PolicyArtifact",
        "ContextArtifact",
        "ExecutionArtifact",
        "ConformanceArtifact",
        "CustomArtifact"
      ]
    },
    "schema_version": { "type": "string" },
    "content_hash": { "$ref": "aira:schema:common:hash:0.1" },
    "content_ref": { "type": "string" },
    "producer_identity": { "$ref": "aira:schema:common:ref:0.1" },
    "provenance_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "dependency_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" }
  }
}
```

---

# 8. Event Descriptor Schema

```json
{
  "$id": "aira:schema:event:event-descriptor:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "event_id",
    "event_type",
    "schema_version",
    "producer_identity",
    "causal_refs",
    "object_refs",
    "artifact_refs",
    "policy_refs",
    "payload_hash",
    "created_at",
    "signature"
  ],
  "properties": {
    "event_id": { "$ref": "aira:schema:common:ref:0.1" },
    "event_type": {
      "type": "string",
      "enum": [
        "ProblemSubmitted",
        "ContextResolved",
        "ReductionCompleted",
        "CapsuleCreated",
        "CapsuleBound",
        "CapsuleCompleted",
        "CapsuleFailed",
        "VerificationCompleted",
        "VerificationFailed",
        "ResultPublished",
        "ArtifactPublished",
        "ArtifactResolved",
        "ArtifactInvalid",
        "ArtifactSuperseded",
        "CapabilityRegistered",
        "PolicyEvaluated",
        "CSURegistered",
        "CSUSuspended",
        "CSUFailed",
        "InvariantViolation",
        "FailureEvidenceCreated",
        "ResearchArtifactCreated",
        "ArtifactPromotionCandidate",
        "CustomEvent"
      ]
    },
    "schema_version": { "type": "string" },
    "producer_identity": { "$ref": "aira:schema:common:ref:0.1" },
    "causal_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "object_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "artifact_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "payload_hash": { "$ref": "aira:schema:common:hash:0.1" },
    "payload_ref": {
      "type": ["string", "null"]
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 9. Protocol Envelope Schema

Використовується всіма Book II protocols.

```json
{
  "$id": "aira:schema:protocol:envelope:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "protocol_id",
    "protocol_version",
    "message_type",
    "message_id",
    "issuer_identity",
    "target_scope",
    "policy_refs",
    "payload_hash",
    "payload_ref",
    "created_at",
    "signature"
  ],
  "properties": {
    "protocol_id": {
      "type": "string",
      "enum": [
        "AIRA-EP",
        "AIRA-AP",
        "AIRA-ID",
        "AIRA-DP",
        "AIRA-CAP",
        "AIRA-CRP",
        "AIRA-FED",
        "AIRA-SET"
      ]
    },
    "protocol_version": { "type": "string" },
    "message_type": { "type": "string" },
    "message_id": { "$ref": "aira:schema:common:ref:0.1" },
    "correlation_id": {
      "type": ["string", "null"]
    },
    "causal_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "issuer_identity": { "$ref": "aira:schema:common:ref:0.1" },
    "target_scope": { "$ref": "aira:schema:common:scope-descriptor:0.1" },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "payload_hash": { "$ref": "aira:schema:common:hash:0.1" },
    "payload_ref": {
      "type": ["string", "null"]
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "expires_at": {
      "type": ["string", "null"],
      "format": "date-time"
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

`expires_at` стосується повідомлення, а не Knowledge. Knowledge не має TTL; змінюється confidence, scope, status і evidence chain.

---

# 10. Protocol Response Schema

```json
{
  "$id": "aira:schema:protocol:response:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "message_id",
    "correlation_id",
    "status",
    "created_at",
    "signature"
  ],
  "properties": {
    "message_id": { "$ref": "aira:schema:common:ref:0.1" },
    "correlation_id": { "type": ["string", "null"] },
    "status": {
      "type": "string",
      "enum": [
        "ACCEPTED",
        "REJECTED",
        "DEFERRED",
        "REQUIRES_POLICY",
        "REQUIRES_EVIDENCE",
        "UNSUPPORTED_VERSION",
        "UNSUPPORTED_CAPABILITY",
        "INVALID_SIGNATURE",
        "INVALID_ARTIFACT",
        "INVARIANT_VIOLATION"
      ]
    },
    "reason_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 11. Identity Descriptor Schema

```json
{
  "$id": "aira:schema:identity:identity-descriptor:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "identity_id",
    "identity_type",
    "public_keys",
    "policy_refs",
    "signature"
  ],
  "properties": {
    "identity_id": { "$ref": "aira:schema:common:ref:0.1" },
    "identity_type": {
      "type": "string",
      "enum": ["user", "csu", "federation", "service", "organization"]
    },
    "public_keys": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["key_id", "algorithm", "public_key_material", "valid_from"],
        "properties": {
          "key_id": { "type": "string" },
          "algorithm": { "type": "string" },
          "public_key_material": { "type": "string" },
          "valid_from": { "$ref": "aira:schema:common:timestamp:0.1" },
          "valid_to": {
            "type": ["string", "null"],
            "format": "date-time"
          }
        }
      }
    },
    "trust_anchors": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "metadata_hash": {
      "type": ["string", "null"]
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 12. Policy Query Schema

```json
{
  "$id": "aira:schema:policy:query:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "subject",
    "action",
    "object_refs",
    "context_refs",
    "evidence_refs",
    "requested_at"
  ],
  "properties": {
    "subject": { "$ref": "aira:schema:common:ref:0.1" },
    "csu_ref": {
      "type": ["string", "null"]
    },
    "action": { "type": "string" },
    "object_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "artifact_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "context_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "requested_at": { "$ref": "aira:schema:common:timestamp:0.1" }
  }
}
```

---

# 13. Policy Decision Schema

```json
{
  "$id": "aira:schema:policy:decision:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": ["decision", "requirements", "reason_refs", "signature"],
  "properties": {
    "decision": {
      "type": "string",
      "enum": ["ALLOW", "DENY", "REQUIRE"]
    },
    "requirements": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true
      }
    },
    "reason_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

Policy ніколи не виконує дію. Вона лише повертає `ALLOW`, `DENY` або `REQUIRE`.

---

# 14. Capability Descriptor Schema

```json
{
  "$id": "aira:schema:capability:descriptor:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "capability_id",
    "capability_type",
    "schema_version",
    "provider_csu",
    "input_artifact_types",
    "output_artifact_types",
    "constraints",
    "scope",
    "policy_refs",
    "signature"
  ],
  "properties": {
    "capability_id": { "$ref": "aira:schema:common:ref:0.1" },
    "capability_type": { "type": "string" },
    "schema_version": { "type": "string" },
    "provider_csu": { "$ref": "aira:schema:common:ref:0.1" },
    "input_artifact_types": {
      "type": "array",
      "items": { "type": "string" }
    },
    "output_artifact_types": {
      "type": "array",
      "items": { "type": "string" }
    },
    "required_context": {
      "type": "array",
      "items": { "type": "string" }
    },
    "constraints": {
      "type": "object",
      "additionalProperties": true,
      "properties": {
        "latency_max_ms": { "type": ["integer", "null"] },
        "cost_max": { "type": ["number", "null"] },
        "privacy_class": { "type": ["string", "null"] },
        "trust_min": { "type": ["number", "null"], "minimum": 0, "maximum": 1 },
        "reliability_min": { "type": ["number", "null"], "minimum": 0, "maximum": 1 }
      }
    },
    "cost_model_ref": {
      "type": ["string", "null"]
    },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "confidence": {
      "type": "number",
      "minimum": 0,
      "maximum": 1
    },
    "scope": { "$ref": "aira:schema:common:scope-descriptor:0.1" },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

Capability описує можливість, а не фізичний вузол, GPU, LLM або конкретну реалізацію.

---

# 15. CSU Manifest Schema

```json
{
  "$id": "aira:schema:csu:manifest:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "csu_id",
    "csu_name",
    "csu_type",
    "csu_version",
    "abi_version",
    "manifest_version",
    "identity_ref",
    "publisher_identity",
    "capabilities",
    "permissions",
    "event_subscriptions",
    "event_outputs",
    "artifact_inputs",
    "artifact_outputs",
    "policy_refs",
    "sandbox",
    "signature",
    "created_at"
  ],
  "properties": {
    "csu_id": { "$ref": "aira:schema:common:ref:0.1" },
    "csu_name": { "type": "string" },
    "csu_type": {
      "type": "string",
      "enum": [
        "Context",
        "Reduction",
        "Evidence",
        "Epistemic",
        "Execution",
        "Verification",
        "Artifact",
        "Discovery",
        "Federation",
        "Settlement",
        "Optimization",
        "PHM",
        "Evolution",
        "Research",
        "HumanInteraction",
        "Custom"
      ]
    },
    "csu_version": { "type": "string" },
    "abi_version": { "type": "string" },
    "manifest_version": { "type": "string" },
    "identity_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "publisher_identity": { "$ref": "aira:schema:common:ref:0.1" },
    "capabilities": {
      "type": "array",
      "items": { "$ref": "aira:schema:capability:descriptor:0.1" }
    },
    "permissions": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true
      }
    },
    "event_subscriptions": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true
      }
    },
    "event_outputs": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true
      }
    },
    "artifact_inputs": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true
      }
    },
    "artifact_outputs": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true
      }
    },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "resource_requirements": {
      "type": "object",
      "additionalProperties": true
    },
    "sandbox": {
      "type": "object",
      "additionalProperties": false,
      "required": ["filesystem", "network", "process", "device_access", "secret_access"],
      "properties": {
        "filesystem": { "type": "string", "enum": ["none", "read_only", "scoped", "full"] },
        "network": { "type": "string", "enum": ["none", "scoped", "full"] },
        "process": { "type": "string", "enum": ["in_process", "subprocess", "wasm", "container", "vm"] },
        "device_access": { "type": "string", "enum": ["none", "scoped", "full"] },
        "secret_access": { "type": "string", "enum": ["none", "scoped", "full"] }
      }
    },
    "lifecycle_hooks": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "init": { "type": "boolean" },
        "activate": { "type": "boolean" },
        "suspend": { "type": "boolean" },
        "resume": { "type": "boolean" },
        "shutdown": { "type": "boolean" }
      }
    },
    "provenance_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" }
  }
}
```

---

# 16. Execution Capsule Schema

```json
{
  "$id": "aira:schema:execution:capsule:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "capsule_id",
    "problem_statement_ref",
    "context_ref",
    "required_capabilities",
    "input_artifact_refs",
    "constraints",
    "policy_refs",
    "provenance_refs",
    "signature"
  ],
  "properties": {
    "capsule_id": { "$ref": "aira:schema:common:ref:0.1" },
    "problem_statement_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "context_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "required_capabilities": {
      "type": "array",
      "items": { "$ref": "aira:schema:capability:descriptor:0.1" }
    },
    "input_artifact_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "constraints": {
      "type": "object",
      "additionalProperties": true
    },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "provenance_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 17. Evidence Artifact Schema

```json
{
  "$id": "aira:schema:evidence:evidence-artifact:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "evidence_id",
    "evidence_type",
    "claim_refs",
    "source_refs",
    "observation",
    "context_refs",
    "provenance_refs",
    "created_at",
    "signature"
  ],
  "properties": {
    "evidence_id": { "$ref": "aira:schema:common:ref:0.1" },
    "evidence_type": {
      "type": "string",
      "enum": [
        "Observation",
        "ExecutionEvidence",
        "VerificationEvidence",
        "FailureEvidence",
        "TelemetryEvidence",
        "HumanReviewEvidence",
        "ExternalEvidence",
        "CounterEvidence"
      ]
    },
    "claim_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "source_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "observation": {
      "type": "object",
      "additionalProperties": true
    },
    "context_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "provenance_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 18. Epistemic Assessment Schema

```json
{
  "$id": "aira:schema:epistemic:assessment:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "assessment_id",
    "claim_ref",
    "evidence_refs",
    "counter_evidence_refs",
    "epistemic_status",
    "confidence",
    "scope",
    "revision_refs",
    "signature"
  ],
  "properties": {
    "assessment_id": { "$ref": "aira:schema:common:ref:0.1" },
    "claim_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "counter_evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "epistemic_status": {
      "type": "string",
      "enum": [
        "Contradicted",
        "Absurd",
        "Myth",
        "Legend",
        "Anecdote",
        "Observation",
        "Assumption",
        "Hypothesis",
        "WorkingModel",
        "ValidatedModel",
        "Theory",
        "EmpiricalFact",
        "ProtocolFact",
        "Axiom"
      ]
    },
    "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
    "scope": { "$ref": "aira:schema:common:scope-descriptor:0.1" },
    "contextual_fitness": {
      "type": ["number", "null"],
      "minimum": 0,
      "maximum": 1
    },
    "revision_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 19. Verified Result Artifact Schema

```json
{
  "$id": "aira:schema:result:verified-result-artifact:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "result_id",
    "problem_statement_ref",
    "context_ref",
    "solution_refs",
    "evidence_refs",
    "verification_status",
    "confidence",
    "scope",
    "provenance_refs",
    "artifact_hash",
    "signature",
    "created_at"
  ],
  "properties": {
    "result_id": { "$ref": "aira:schema:common:ref:0.1" },
    "problem_statement_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "context_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "solution_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "verification_status": {
      "type": "string",
      "enum": [
        "VERIFIED",
        "REJECTED",
        "PARTIAL",
        "INSUFFICIENT_EVIDENCE",
        "NEGATIVE_RESULT",
        "OPEN_RESEARCH"
      ]
    },
    "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
    "scope": { "$ref": "aira:schema:common:scope-descriptor:0.1" },
    "provenance_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "artifact_hash": { "$ref": "aira:schema:common:hash:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" }
  }
}
```

---

# 20. Differentiated Solution Field Schema

Використовується, коли Solution Field не звужується до однієї області, а розщеплюється на кілька рівноправних alternatives.

```json
{
  "$id": "aira:schema:solution:differentiated-field:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "field_id",
    "problem_statement_ref",
    "context_ref",
    "alternatives",
    "requires_human_collapse",
    "evidence_refs",
    "created_at",
    "signature"
  ],
  "properties": {
    "field_id": { "$ref": "aira:schema:common:ref:0.1" },
    "problem_statement_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "context_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "alternatives": {
      "type": "array",
      "minItems": 2,
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": [
          "alternative_id",
          "description",
          "assumptions",
          "risks",
          "consequences",
          "evidence_refs"
        ],
        "properties": {
          "alternative_id": { "$ref": "aira:schema:common:ref:0.1" },
          "description": { "type": "string" },
          "assumptions": {
            "type": "array",
            "items": { "type": "string" }
          },
          "risks": {
            "type": "array",
            "items": { "type": "string" }
          },
          "consequences": {
            "type": "array",
            "items": { "type": "string" }
          },
          "evidence_refs": {
            "type": "array",
            "items": { "$ref": "aira:schema:common:ref:0.1" }
          }
        }
      }
    },
    "requires_human_collapse": { "type": "boolean", "const": true },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 21. Human Choice Artifact Schema

```json
{
  "$id": "aira:schema:human:choice-artifact:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "choice_id",
    "solution_field_ref",
    "selected_alternative_ref",
    "decision_maker_identity",
    "decision_context_refs",
    "created_at",
    "signature"
  ],
  "properties": {
    "choice_id": { "$ref": "aira:schema:common:ref:0.1" },
    "solution_field_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "selected_alternative_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "decision_maker_identity": { "$ref": "aira:schema:common:ref:0.1" },
    "decision_context_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 22. Failure Event Payload Schema

```json
{
  "$id": "aira:schema:failure:event-payload:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "failure_type",
    "severity",
    "message",
    "object_refs",
    "artifact_refs",
    "evidence_refs",
    "recoverable"
  ],
  "properties": {
    "failure_type": {
      "type": "string",
      "enum": [
        "InputArtifactInvalid",
        "PolicyDenied",
        "CapabilityUnavailable",
        "ExecutionFailed",
        "VerificationFailed",
        "Timeout",
        "BudgetExceeded",
        "InvariantViolation",
        "CSUInternalError",
        "DependencyUnavailable",
        "ProtocolRejected",
        "SignatureInvalid"
      ]
    },
    "severity": {
      "type": "string",
      "enum": ["info", "warning", "error", "critical"]
    },
    "message": { "type": "string" },
    "object_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "artifact_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "recoverable": { "type": "boolean" }
  }
}
```

Failure не є порожнім результатом. Failure створює Event і, за можливості, Evidence Artifact.

---

# 23. Research Artifact Schema

```json
{
  "$id": "aira:schema:research:research-artifact:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "artifact_id",
    "research_domain",
    "hypothesis_refs",
    "evidence_refs",
    "counter_evidence_refs",
    "method_ref",
    "experiment_refs",
    "result_status",
    "confidence",
    "scope",
    "risk_descriptor",
    "policy_refs",
    "provenance_refs",
    "created_at",
    "signature"
  ],
  "properties": {
    "artifact_id": { "$ref": "aira:schema:common:ref:0.1" },
    "research_domain": { "type": "string" },
    "hypothesis_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "counter_evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "method_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "experiment_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "result_status": {
      "type": "string",
      "enum": [
        "proposed",
        "running",
        "replicated",
        "failed",
        "contradicted",
        "validated",
        "promoted_candidate",
        "deprecated"
      ]
    },
    "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
    "scope": { "$ref": "aira:schema:common:scope-descriptor:0.1" },
    "risk_descriptor": {
      "type": "object",
      "additionalProperties": true
    },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "provenance_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 24. Artifact Promotion Candidate Schema

```json
{
  "$id": "aira:schema:research:promotion-candidate:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "promotion_id",
    "source_artifact_ref",
    "evidence_refs",
    "validation_refs",
    "compatibility_check_ref",
    "policy_refs",
    "rollback_path_ref",
    "promotion_status",
    "created_at",
    "signature"
  ],
  "properties": {
    "promotion_id": { "$ref": "aira:schema:common:ref:0.1" },
    "source_artifact_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "evidence_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "validation_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "compatibility_check_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "rollback_path_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "promotion_status": {
      "type": "string",
      "enum": ["candidate", "canary", "accepted", "rejected", "rolled_back"]
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 25. Settlement Receipt Schema

```json
{
  "$id": "aira:schema:settlement:receipt:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "receipt_id",
    "execution_or_artifact_ref",
    "provider_identity",
    "consumer_identity",
    "capability_refs",
    "contribution_descriptor",
    "cost_descriptor_ref",
    "verification_refs",
    "policy_refs",
    "created_at",
    "signature"
  ],
  "properties": {
    "receipt_id": { "$ref": "aira:schema:common:ref:0.1" },
    "execution_or_artifact_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "provider_identity": { "$ref": "aira:schema:common:ref:0.1" },
    "consumer_identity": { "$ref": "aira:schema:common:ref:0.1" },
    "capability_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "contribution_descriptor": {
      "type": "object",
      "additionalProperties": false,
      "required": ["amount", "unit", "method"],
      "properties": {
        "amount": { "type": ["number", "null"] },
        "unit": { "type": ["string", "null"] },
        "method": { "type": "string" }
      }
    },
    "cost_descriptor_ref": { "$ref": "aira:schema:common:ref:0.1" },
    "verification_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "policy_refs": {
      "type": "array",
      "items": { "$ref": "aira:schema:common:ref:0.1" }
    },
    "created_at": { "$ref": "aira:schema:common:timestamp:0.1" },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

Settlement Receipt не повинен містити raw prompt, private payload або secret material.

---

# 26. Conformance Report Schema

```json
{
  "$id": "aira:schema:conformance:report:0.1",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "implementation",
    "aira",
    "results",
    "failures",
    "signature"
  ],
  "properties": {
    "implementation": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name", "version"],
      "properties": {
        "name": { "type": "string" },
        "version": { "type": "string" },
        "commit": { "type": ["string", "null"] }
      }
    },
    "aira": {
      "type": "object",
      "additionalProperties": false,
      "required": ["standard_version", "profile"],
      "properties": {
        "standard_version": { "type": "string" },
        "profile": {
          "type": "string",
          "enum": ["C0", "C1", "C2", "C3", "C4", "C5"]
        }
      }
    },
    "results": {
      "type": "object",
      "additionalProperties": false,
      "required": ["total", "passed", "failed", "skipped", "unsupported", "invalid"],
      "properties": {
        "total": { "type": "integer", "minimum": 0 },
        "passed": { "type": "integer", "minimum": 0 },
        "failed": { "type": "integer", "minimum": 0 },
        "skipped": { "type": "integer", "minimum": 0 },
        "unsupported": { "type": "integer", "minimum": 0 },
        "invalid": { "type": "integer", "minimum": 0 }
      }
    },
    "failures": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["test_id", "reason", "evidence_refs"],
        "properties": {
          "test_id": { "type": "string" },
          "reason": { "type": "string" },
          "evidence_refs": {
            "type": "array",
            "items": { "$ref": "aira:schema:common:ref:0.1" }
          }
        }
      }
    },
    "signature": { "$ref": "aira:schema:common:signature:0.1" }
  }
}
```

---

# 27. Schema Pack Validation Rules

## 27.1 Required

Кожна реалізація AIRA-C0+ **MUST** підтримувати:

```text
Core Object Descriptor
Artifact Descriptor
Event Descriptor
Policy Query
Policy Decision
Verified Result Artifact
```

## 27.2 Required for C1+

```text
CSU Manifest
Capability Descriptor
Execution Capsule
Evidence Artifact
Epistemic Assessment
```

## 27.3 Required for C2+

```text
Protocol Envelope
Protocol Response
Identity Descriptor
Discovery / Capability messages
```

## 27.4 Required for C3+

```text
Federation Descriptor
Capability Advertisement
CRP Route Request / Candidate
```

## 27.5 Required for C4+

```text
Settlement Receipt
Audit Event
Contribution Record
```

## 27.6 Required for C5+

```text
Research Artifact
Promotion Candidate
Open Research Artifact
Negative Result Artifact
Human Choice Artifact
```

---

# 28. Forbidden Schema Couplings

Schema Pack **MUST NOT** introduce fields that require:

```text
gpu_id
node_id as core identity
driver_id as canonical role
scheduler_id
wallet_address as mandatory identity
blockchain_height as mandatory settlement field
llm_model_id as core dependency
global_state_version
```

Такі поля можуть існувати тільки як implementation-specific metadata або extension fields у відповідних Artifact / CSU / Protocol schemas.

---

# 29. Extension Rules

Schema extension **MUST** be:

```text
versioned;
namespaced;
policy-visible;
backward-compatible unless major version changes;
covered by conformance tests if normative;
documented as Reference or Research if non-normative.
```

Extension namespace format:

```text
x-<organization-or-federation>-<field-name>
```

Example:

```json
{
  "x-lab42-energy_score": 0.83
}
```

Extension поля не можуть змінювати значення нормативних полів.

---

# 30. Canonical Schema Registry

Schema Pack v0.1 фіксує мінімальний registry:

```text
aira:schema:common:identifier:0.1
aira:schema:common:ref:0.1
aira:schema:common:hash:0.1
aira:schema:common:timestamp:0.1
aira:schema:common:signature:0.1
aira:schema:common:scope-descriptor:0.1

aira:schema:core:object-descriptor:0.1
aira:schema:core:problem-statement:0.1

aira:schema:artifact:artifact-descriptor:0.1
aira:schema:artifact:context-artifact:0.1

aira:schema:event:event-descriptor:0.1

aira:schema:protocol:envelope:0.1
aira:schema:protocol:response:0.1

aira:schema:identity:identity-descriptor:0.1

aira:schema:policy:query:0.1
aira:schema:policy:decision:0.1

aira:schema:capability:descriptor:0.1
aira:schema:csu:manifest:0.1

aira:schema:execution:capsule:0.1

aira:schema:evidence:evidence-artifact:0.1
aira:schema:epistemic:assessment:0.1

aira:schema:result:verified-result-artifact:0.1

aira:schema:solution:differentiated-field:0.1
aira:schema:human:choice-artifact:0.1

aira:schema:failure:event-payload:0.1

aira:schema:research:research-artifact:0.1
aira:schema:research:promotion-candidate:0.1

aira:schema:settlement:receipt:0.1

aira:schema:conformance:report:0.1
```

---

# 31. Minimal Implementation Requirement

Мінімальна реалізація AIRA повинна валідувати щонайменше:

```text
Problem Statement
Context Artifact
Execution Capsule
Event Descriptor
Artifact Descriptor
CSU Manifest
Capability Descriptor
Policy Query / Decision
Verified Result Artifact
Conformance Report
```

Без цього неможливо автоматизувати Conformance Test Specification.

---

# 32. Status

AIRA Schema Pack v0.1 формалізує data contract layer.

Він закріплює:

```text
canonical references;
immutable descriptors;
signed objects;
event causality;
artifact provenance;
policy decisions;
capability declarations;
CSU manifests;
verified result structure;
failure-to-evidence semantics;
research isolation;
conformance reports.
```

Schema Pack не змінює мету AIRA. Він лише робить її машинно перевірною:

```text
Problem Statement
↓
Progressive Resolution
↓
Verified Result Artifact
↓
Evidence
↓
Evolution
↓
Lower-cost future Resolution
```

Наступний нормативний документ:

```text
AIRA RFC Template & Change Process v0.1
```

Scope:

```text
RFC classes;
RFC metadata;
architecture-change criteria;
protocol-change criteria;
CSU-contract-change criteria;
schema-change criteria;
research-promotion process;
compatibility impact;
rollback requirements;
conformance-test requirements.
```
