# Book II — Protocol Specifications (аналіз)

**Джерело:** `Manifesto etc/Book II — Protocol Specifications v0.1.md`  
**Depends on:** Book 0, Book I  
**Формат:** structured digest §1–25 після повного читання  
**Confidence:** High (Evidence); Medium (Inference)

---

## 1. Роль і позиція

**Evidence (§0–3):** Normative Protocol Layer; між Core ABI і CSU contracts.  
**Evidence (§0 Exports):** Protocol Envelope + Discovery, CRP, Artifact, Event, Identity, Federation, Settlement/Audit (7 named protocols + envelope).  
**Evidence (§24):** Canonical set додає AIRA-CAP → **8** wire protocols (EP, AP, ID, DP, CAP, CRP, FED, SET).  
**Evidence (§2):** MUST NOT: routing/scheduler/optimizer/PHM/ML/knowledge algorithms/blockchain consensus/transport/storage implementations → Book IV/V.

**Inference:** Book II = interoperability contracts, не «як саме маршрутизувати». Exports (§0) і Canonical Set (§24) різняться на CAP — CAP формалізований у §9 і §24.

---

## 2. Common principles, envelope, responses

**Evidence (§5):** No direct CSU-to-CSU mutation; no shared mutable/hidden global state; **no domain-specific semantics in Core**; policy before action; artifact immutability; event traceability; identity verification; provenance.

**Evidence (§6):** Common message envelope (protocol_id/version, message_type/id, correlation, causal_refs, issuer, target_scope, policy_refs, payload_hash/ref, timestamps, signature).  
**Evidence (§6.1):** `expires_at` = message TTL, **not** Knowledge TTL.

**Evidence (§7):** Response set ACCEPTED…INVARIANT_VIOLATION; rejection MUST yield Event or Evidence Artifact.

---

## 3. Discovery (§8)

**Evidence:** Шукає Capability/CSU, не Node; Query/Response schemas; MUST NOT expose topology by default; MUST NOT require global registry; MAY DHT/registry/gossip/etc.

**Inference:** Pivot від Meditation_About/10 (resource/overlay) → capability discovery.

---

## 4. Capability Advertisement (§9)

**Evidence:** Advertisement schema; MUST describe ability not hardware; policy + evidence; revocable; SHOULD cost model; MAY PHM health metadata.  
Capability = reduce Solution Uncertainty / materialize solution part.

---

## 5. CRP (§10)

**Evidence:** Route by Capability not Node; Route Request/Candidate; MUST NOT single global table; MUST NOT expose private topology unless policy; multiple candidates; Policy Gate before bind; Events for select/reject/fail.

**Inference:** Зберігає ідею Meditation_About/15 (capsule routing), прибирає обов’язковий algorithm.

---

## 6. Artifact Protocol (§11)

**Evidence:** Publish/Resolve/Response; MUST immutability, provenance, hash, dependency graph, supersession without mutation, revocation metadata without deleting history; MUST NOT one global storage.

---

## 7. Event Protocol (§12)

**Evidence:** Міжсистемний формат поверх Book I Event Runtime; Publish/Subscription; MUST causal_refs, idempotent delivery, duplicate detection; MUST NOT total global ordering; local ordering in causal scope; policy-controlled propagation.

---

## 8. Identity (§13)

**Evidence:** identity_type user|csu|federation|service|organization; keys; trust anchors; MUST pseudonymous; MUST NOT require real-world ID disclosure; key rotation/revocation; federation-scoped; artifact/event signatures.

**Inference:** Wallet/Node/Resource не fundamental (на відміну від Meditation_About/9 early model).

---

## 9. Federation (§14)

**Evidence:** Descriptor/Join/Response; MUST autonomy; MUST NOT force global policy uniformity; limited participation; policy-based export/import; exit/revocation.

---

## 10. Settlement / Audit (§15)

**Evidence:** Receipts; MUST NOT expose prompt/private payload; verifiable receipts/hashes only; non-fiat contribution; own-resource without forced redistribution; MAY blockchain OR append-only/signed ledger/local audit.  
**Evidence:** Blockchain не обов’язковий.

---

## 11. Versioning, extensions, policy, privacy, failure

**Evidence (§16–20):** Negotiation → UNSUPPORTED_VERSION без side effects; extensions explicit/versioned/policy-visible; every action through Policy Gate; MUST NOT leak raw prompt/docs/outputs/keys; failure → Event (ProtocolRejected…VerificationFailed) usable as Evidence.

---

## 12. Conformance & canonical set

**Evidence (§21):** P0 Event+Artifact+Identity; P1 +Discovery+CAP+CRP; P2 +Federation; P3 +Settlement.  
**Evidence (§22):** Envelope, signatures, immutability, causality, policy, versioning, no domain in Core, failure events.  
**Evidence (§23 Exclusions):** exact DHT/gossip/route scoring/PHM/settlement ledger/storage/optimizer/cost formula/ML — не в Book II (→ IV/V).  
**Evidence (§24):** AIRA-EP, AP, ID, DP, CAP, CRP, FED, SET.

**Evidence (§25):** Next = Book III CSU contracts.

**Inference / Unknown:** Mapping P0–P3 ↔ SCD AIRA-C0…C5 / Book I L0–L3 — див. TF3-001.
