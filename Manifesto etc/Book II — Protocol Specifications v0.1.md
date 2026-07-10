# Book II — Protocol Specifications v0.1

```text
Type: Normative
Scope: AIRA Protocol Layer
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
Exports:
  - Protocol Envelope
  - Discovery Protocol
  - Capability Routing Protocol
  - Artifact Protocol
  - Event Protocol
  - Identity Protocol
  - Federation Protocol
  - Settlement/Audit Protocol
```

Book II описує **мережеві та між-CSU контракти** AIRA.

Book II не описує алгоритми, евристики, ML, routing heuristics, PHM prediction, storage implementation або конкретну Reference Implementation. Це відповідає Book I, де Core обмежений Object Model, Stable ABI, Event Runtime, Policy Gate, Invariant Checker, Security Boundary і CSU Runtime. 

---

# 1. Scope

Book II визначає, як незалежні реалізації AIRA обмінюються:

```text
Capabilities
Artifacts
Events
Identity claims
Federation metadata
Settlement receipts
Routing intents
```

Book II гарантує **interoperability**, але не гарантує однакові внутрішні алгоритми.

---

# 2. Non-Goals

Book II **MUST NOT** визначати:

```text
routing algorithm
scheduler
optimizer
PHM model
ML model
knowledge aggregation algorithm
blockchain consensus
transport implementation
storage engine
```

Будь-яка така логіка належить до Book IV або Book V.

---

# 3. Protocol Layer Position

Book II знаходиться між Core ABI та CSU contracts.

```text
Book 0 — Laws / Ontology / Invariants
        ↓
Book I — Core / ABI / Object Runtime
        ↓
Book II — Protocol Contracts
        ↓
Book III — CSU Contracts
        ↓
Book IV — Reference Implementation
        ↓
Book V — Research
```

Ранні документи описували окремі протоколи як частину P2P/AI Compute Network, зокрема Discovery, CRP, Identity/Federation і Blockchain/Settlement. У Book II вони нормалізуються як **контракти**, а не як конкретні алгоритмічні реалізації.    

---

# 4. Normative Keywords

У цьому документі:

```text
MUST      — обов'язкова вимога;
MUST NOT  — заборона;
SHOULD    — рекомендована вимога;
MAY       — дозволена опція.
```

---

# 5. Common Protocol Principles

Усі протоколи Book II **MUST** дотримуватись таких принципів:

```text
No direct CSU-to-CSU mutation
No shared mutable state
No hidden global state
No domain-specific semantics in Core
Policy before action
Artifact immutability
Event traceability
Identity verification
Provenance preservation
```

---

# 6. Common Message Envelope

Кожне міжсистемне повідомлення Book II **MUST** мати envelope.

```yaml
protocol_id: string
protocol_version: string
message_type: string
message_id: string
correlation_id: string | null
causal_refs: [event_ref]
issuer_identity: identity_ref
target_scope: scope_descriptor
policy_refs: [policy_ref]
payload_hash: hash
payload_ref: artifact_ref | inline_payload_ref
created_at: timestamp
expires_at: timestamp | null
signature: signature
```

## 6.1 Пояснення

`expires_at` стосується **повідомлення**, а не знання. Це не TTL Knowledge. Knowledge не має TTL; еволюціонує лише confidence, scope, evidence і epistemic status.  

---

# 7. Common Response Model

Усі протоколи **MUST** повертати один із результатів:

```text
ACCEPTED
REJECTED
DEFERRED
REQUIRES_POLICY
REQUIRES_EVIDENCE
UNSUPPORTED_VERSION
UNSUPPORTED_CAPABILITY
INVALID_SIGNATURE
INVALID_ARTIFACT
INVARIANT_VIOLATION
```

Відмова не є "порожнім" результатом. Вона **MUST** породити Event або Evidence Artifact.

---

# 8. Discovery Protocol

## 8.1 Purpose

Discovery Protocol дозволяє знайти не вузол, а доступну **Capability** або CSU, здатний брати участь у Progressive Resolution.

Рання модель Discovery шукала ресурси, вузли або DHT-індекси. У канонічній моделі шукається Capability / CSU, а Node є реалізаційною деталлю. 

---

## 8.2 Discovery Query

```yaml
query_id: string
requested_capability: capability_descriptor
context_ref: context_ref | null
policy_refs: [policy_ref]
required_evidence: [evidence_ref]
constraints:
  latency_max: duration | null
  cost_max: cost_value | null
  trust_min: number | null
  region_scope: scope_descriptor | null
  privacy_class: string | null
issuer_identity: identity_ref
signature: signature
```

---

## 8.3 Discovery Response

```yaml
query_id: string
candidate_capabilities:
  - capability_ref
    provider_csu: csu_ref
    scope: scope_descriptor
    confidence: number
    evidence_refs: [evidence_ref]
    cost_model_ref: artifact_ref
    policy_refs: [policy_ref]
response_signature: signature
```

---

## 8.4 Requirements

Discovery Protocol:

```text
MUST NOT expose internal topology by default.
MUST NOT require global registry.
MUST support local, federation, and inter-federation discovery.
MUST allow multiple implementations.
MAY use DHT, registry, gossip, address book, federation gateway, or static bootstrap.
```

---

# 9. Capability Advertisement Protocol

## 9.1 Purpose

Capability Advertisement дозволяє CSU або Federation оголошувати, які можливості вони можуть надати.

---

## 9.2 Capability Advertisement

```yaml
advertisement_id: string
provider_csu: csu_ref
capability:
  capability_type: string
  schema_version: string
  constraints: object
  scope: scope_descriptor
  cost_model_ref: artifact_ref
  evidence_refs: [evidence_ref]
  confidence: number
policy_refs: [policy_ref]
validity_window:
  from: timestamp
  to: timestamp | null
signature: signature
```

---

## 9.3 Requirements

Capability Advertisement:

```text
MUST describe ability, not hardware.
MUST include policy scope.
MUST include provenance or evidence.
MUST be revocable.
SHOULD include cost model.
MAY include health/risk metadata if supplied by PHM CSU.
```

Capability означає **будь-яку можливість зменшити Solution Uncertainty або матеріалізувати частину рішення**, а не лише compute hardware.

---

# 10. Capability Routing Protocol (CRP)

## 10.1 Purpose

CRP визначає, як знайти допустиму композицію Capability для Execution Capsule або іншого Resolution step.

Рання CRP-модель вже визначала маршрутизацію не IP-пакетів, а Execution Capsule через Capability Query; Book II зберігає цю ідею, але прибирає обов’язкову залежність від конкретного routing algorithm. 

---

## 10.2 CRP Route Request

```yaml
route_request_id: string
capsule_ref: execution_capsule_ref
required_capabilities: [capability_descriptor]
context_ref: context_ref
policy_refs: [policy_ref]
constraints:
  cost_max: cost_value | null
  latency_max: duration | null
  reliability_min: number | null
  privacy_class: string | null
  federation_scope: scope_descriptor | null
issuer_identity: identity_ref
signature: signature
```

---

## 10.3 CRP Route Candidate

```yaml
route_candidate_id: string
route_request_id: string
capability_chain:
  - capability_ref
    provider_csu: csu_ref
    role: string
    policy_refs: [policy_ref]
estimated_cost: cost_descriptor
risk_descriptor_ref: artifact_ref | null
evidence_refs: [evidence_ref]
confidence: number
signature: signature
```

---

## 10.4 CRP Requirements

CRP:

```text
MUST route by Capability, not Node.
MUST NOT require a single global routing table.
MUST NOT expose private federation topology unless policy allows it.
MUST support multiple equivalent route candidates.
MUST allow Policy Gate evaluation before binding.
MUST produce Event records for route selection, rejection, and failure.
```

---

# 11. Artifact Protocol

## 11.1 Purpose

Artifact Protocol визначає, як Artifact публікуються, перевіряються, реплікуються, відкликаються та посилаються один на одного.

Artifact Lifecycle був сформований як універсальна модель для Knowledge Package, Driver, Context Profile, PHM Model, Routing Heuristic, Policy Package та інших результатів системи. 

---

## 11.2 Artifact Publish

```yaml
artifact_id: string
artifact_type: string
schema_version: string
content_hash: hash
content_ref: storage_ref
producer_identity: identity_ref
provenance_refs: [artifact_or_event_ref]
dependency_refs: [artifact_ref]
policy_refs: [policy_ref]
signature: signature
created_at: timestamp
```

---

## 11.3 Artifact Resolve

```yaml
artifact_ref: artifact_ref
required_hash: hash | null
required_type: string | null
required_policy_refs: [policy_ref]
requester_identity: identity_ref
signature: signature
```

---

## 11.4 Artifact Response

```yaml
artifact_ref: artifact_ref
artifact_descriptor: artifact_descriptor
availability:
  local: boolean
  remote_sources: [source_ref]
verification_status: enum
policy_status: enum
signature: signature
```

---

## 11.5 Requirements

Artifact Protocol:

```text
MUST preserve immutability.
MUST preserve provenance.
MUST verify content hash.
MUST support dependency graph.
MUST support supersession without mutation.
MUST support revocation metadata without deleting historical artifacts.
MUST NOT require one global storage backend.
```

---

# 12. Event Protocol

## 12.1 Purpose

Event Protocol визначає між-CSU та між-federation поширення подій.

Core Event Runtime описано в Book I. Book II визначає сумісний міжсистемний формат подій.

---

## 12.2 Event Publish

```yaml
event_id: string
event_type: string
schema_version: string
producer_identity: identity_ref
causal_refs: [event_ref]
object_refs: [object_ref]
artifact_refs: [artifact_ref]
policy_refs: [policy_ref]
payload_hash: hash
payload_ref: artifact_ref | inline_payload_ref
created_at: timestamp
signature: signature
```

---

## 12.3 Event Subscription

```yaml
subscription_id: string
event_filter:
  event_types: [string]
  object_refs: [object_ref]
  artifact_types: [string]
  scope: scope_descriptor
  policy_refs: [policy_ref]
subscriber_identity: identity_ref
signature: signature
```

---

## 12.4 Event Requirements

Event Protocol:

```text
MUST preserve causal_refs.
MUST support idempotent delivery.
MUST support duplicate detection.
MUST NOT require total global ordering.
MUST support local ordering within causal scope.
MUST allow policy-controlled propagation.
MUST distinguish control events from observation events.
```

---

# 13. Identity Protocol

## 13.1 Purpose

Identity Protocol визначає, як User, CSU, Federation та Artifact Producer ідентифікуються та підписують об'єкти.

Рання модель Identity/Trust/Federation містила Human, User, Wallet, Node і Resource identity. У канонічній моделі Book II зберігає криптографічну ідентичність, але не робить Wallet/Node/Resource фундаментальними сутностями. 

---

## 13.2 Identity Descriptor

```yaml
identity_id: string
identity_type: enum  # user | csu | federation | service | organization
public_keys:
  - key_id: string
    algorithm: string
    public_key_material: string
    valid_from: timestamp
    valid_to: timestamp | null
trust_anchors: [identity_ref]
policy_refs: [policy_ref]
metadata_hash: hash | null
signature: signature
```

---

## 13.3 Identity Requirements

Identity Protocol:

```text
MUST support pseudonymous identities.
MUST NOT require public disclosure of real-world identity.
MUST support key rotation.
MUST support revocation.
MUST support federation-scoped identity.
MUST support artifact and event signatures.
```

---

# 14. Federation Protocol

## 14.1 Purpose

Federation Protocol визначає взаємодію автономних сегментів AIRA.

Federation може представляти:

```text
individual deployment
home cluster
organization
university
regional network
public federation
private federation
```

---

## 14.2 Federation Descriptor

```yaml
federation_id: string
federation_type: string
identity_ref: identity_ref
policy_refs: [policy_ref]
supported_protocols:
  - protocol_id: string
    protocol_version: string
capability_summary_ref: artifact_ref | null
trust_policy_ref: policy_ref
gateway_refs: [gateway_ref]
signature: signature
```

---

## 14.3 Federation Join Request

```yaml
request_id: string
requester_identity: identity_ref
target_federation: federation_ref
declared_capabilities: [capability_descriptor]
policy_acceptance_refs: [policy_ref]
evidence_refs: [evidence_ref]
signature: signature
```

---

## 14.4 Federation Response

```yaml
request_id: string
decision: enum  # accepted | rejected | limited | requires_evidence
assigned_scope: scope_descriptor | null
required_policies: [policy_ref]
reason_refs: [artifact_or_event_ref]
signature: signature
```

---

## 14.5 Requirements

Federation Protocol:

```text
MUST preserve federation autonomy.
MUST NOT force global policy uniformity.
MUST allow local policies.
MUST support limited participation.
MUST support policy-based export/import of Capability, Artifact, and Event.
MUST support federation exit and revocation.
```

---

# 15. Settlement / Audit Protocol

## 15.1 Purpose

Settlement Protocol фіксує внесок, використання ресурсів, receipts, contribution records і audit trails.

Book II не вимагає blockchain. Ранні документи описували Neurocoin і Blockchain як службові механізми обліку та довіри; у канонічній моделі це стає optional Settlement/Audit Protocol або CSU.  

---

## 15.2 Settlement Receipt

```yaml
receipt_id: string
execution_or_artifact_ref: artifact_or_event_ref
provider_identity: identity_ref
consumer_identity: identity_ref
capability_refs: [capability_ref]
contribution_descriptor:
  amount: number | null
  unit: string | null
  method: string
cost_descriptor_ref: artifact_ref
verification_refs: [artifact_or_event_ref]
policy_refs: [policy_ref]
created_at: timestamp
signature: signature
```

---

## 15.3 Settlement Requirements

Settlement Protocol:

```text
MUST NOT expose prompt content.
MUST NOT expose private result payload.
MUST record only verifiable receipts, hashes, references, and policy-safe metadata.
MUST support non-fiat contribution accounting.
MUST support own-resource execution without forced redistribution.
MUST support cold-start policy if federation enables it.
MAY use blockchain, append-only log, signed ledger, federation ledger, or local audit store.
```

---

# 16. Protocol Version Negotiation

Кожен учасник **MUST** підтримувати protocol negotiation.

```yaml
supported_protocols:
  - protocol_id: string
    versions: [string]
    extensions: [string]
    required: boolean
```

Якщо версії несумісні:

```text
UNSUPPORTED_VERSION
```

має бути повернено без side effects.

---

# 17. Protocol Extension Model

Розширення протоколу **MUST** бути:

```text
explicit
versioned
policy-visible
backward compatible або clearly breaking
documented
testable
```

Extension не може змінювати Core Object Model без переходу до нової major версії стандарту.

---

# 18. Policy Integration

Кожен protocol action **MUST** проходити Policy Gate.

Protocol message може бути відхилене, якщо:

```text
issuer_identity не має права;
artifact policy не дозволяє доступ;
federation policy забороняє export/import;
capability policy не дозволяє використання;
settlement policy не виконана;
privacy class несумісний.
```

---

# 19. Privacy Requirements

Book II протоколи **MUST** мінімізувати розкриття даних.

Протоколи **MUST NOT** передавати:

```text
raw prompt
raw private document
raw model output
private execution payload
secret keys
unredacted user metadata
```

якщо це прямо не дозволено Policy.

---

# 20. Failure Semantics

Будь-яка protocol failure **MUST** створити Event.

Допустимі failure classes:

```text
ProtocolRejected
PolicyDenied
CapabilityUnavailable
ArtifactUnavailable
IdentityInvalid
SignatureInvalid
FederationRejected
SettlementFailed
RouteFailed
VerificationFailed
```

Failure має бути придатним для Evidence/Evolution Plane.

---

# 21. Protocol Conformance

## Level P0 — Local Protocol Compatibility

Підтримує:

```text
Event Protocol
Artifact Protocol
Identity Protocol
```

## Level P1 — Capability Network Compatibility

Додає:

```text
Discovery Protocol
Capability Advertisement
CRP
```

## Level P2 — Federation Compatibility

Додає:

```text
Federation Protocol
policy-scoped import/export
cross-federation identity
```

## Level P3 — Settlement Compatibility

Додає:

```text
Settlement/Audit Protocol
receipts
contribution accounting
audit proofs
```

---

# 22. Book II Compliance

Реалізація сумісна з Book II, якщо вона:

```text
MUST use common message envelope;
MUST validate signatures;
MUST preserve artifact immutability;
MUST preserve event causality;
MUST enforce policy checks;
MUST support protocol versioning;
MUST expose no domain-specific requirement in Core;
MUST produce valid failure events.
```

---

# 23. Book II Exclusions

Book II не включає:

```text
exact DHT design
exact gossip algorithm
exact route scoring
exact PHM algorithm
exact settlement ledger implementation
exact storage backend
exact optimizer
exact cost formula
exact ML model
```

Ці питання належать до Book IV або Book V.

---

# 24. Canonical Protocol Set

Book II v0.1 фіксує такий мінімальний protocol set:

```text
AIRA-EP   Event Protocol
AIRA-AP   Artifact Protocol
AIRA-ID   Identity Protocol
AIRA-DP   Discovery Protocol
AIRA-CAP  Capability Advertisement Protocol
AIRA-CRP  Capability Routing Protocol
AIRA-FED  Federation Protocol
AIRA-SET  Settlement / Audit Protocol
```

---

# 25. Status

Book II v0.1 формалізує протокольний шар AIRA.

Він закріплює:

```text
transport-agnostic message envelope;
capability-based discovery;
capability routing;
artifact exchange;
event exchange;
identity signatures;
federation interoperability;
settlement/audit receipts;
protocol versioning;
policy-aware communication.
```

Наступний нормативний документ:

```text
Book III — CSU & ABI Contracts v0.1
```

Book III описує:

```text
CSU role contracts
CSU lifecycle contracts
CSU manifest schema
CSU capability declarations
Context CSU
Evidence CSU
Execution CSU
Verification CSU
Artifact CSU
Discovery CSU
Settlement CSU
Optimization CSU
Research CSU
```

Book III не має права описувати конкретні алгоритми; лише сумісні контракти CSU.
