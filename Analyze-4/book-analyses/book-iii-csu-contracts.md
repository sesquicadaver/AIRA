# Book III — CSU & ABI Contracts (аналіз)

**Джерело:** `Manifesto etc/Book III — CSU & ABI Contracts v0.1.md`  
**Depends on:** Book 0, I, II  
**Формат:** structured digest §1–39 після повного читання (1461 рядків)  
**Confidence:** High (Evidence); Medium (Inference)

---

## 1. Роль

**Evidence (§0–1):** Normative CSU Layer; exports contract model, manifest, lifecycle, capability/event/artifact contracts, standard roles.  
**Evidence (§1):** Не алгоритми ML/PHM/routing/storage/transport/ledger/reference impl → IV/V.  
**Evidence (§0):** `Driver` застарілий → `CSU`.

**Inference:** Book III завершує нормативний стандарт (разом з 0–II) для сумісних реалізацій.

---

## 2. CSU definition & principles

**Evidence (§2.1):** CSU = isolated module: Identity, Capability, Stable ABI, Events, Artifacts, Policy Gate, no Core mutation, replaceable; only extension form outside Core.  
**Evidence (§2.2):** NOT kernel/global scheduler/shared mutable/trusted monolith/direct peer dep/physical node/GPU/LLM/blockchain SC.  
**Evidence (§4):** single responsibility; no CSU-to-CSU mutation; no shared mutable state; event/artifact mediation; policy-before-action; least privilege; replaceability; explicit capabilities; failure transparency; provenance.

---

## 3. Manifest & Lifecycle

**Evidence (§5):** Full YAML manifest (id/name/type/versions/ABI, identity, capabilities, permissions, event in/out, artifact in/out, policy, resources, sandbox, lifecycle hooks, provenance, signature).  
**Evidence (§5.1):** MUST signed, declare ABI/permissions/capabilities/artifact types/subscriptions; immutable; supersede via new version.  
**Evidence (§6):** Discovered→Registered→Verified→Active→Suspended→Revoked→Archived (+ 6.1–6.7 semantics). Verified ≠ optimal — лише safe via ABI / no Core invariant break. Revocation MUST Event.

---

## 4. ABI / Capability / Event / Artifact / Policy contracts

**Evidence (§7):** MAY use Book I ABI ops listed; MUST NOT read Core memory, mutate objects/artifacts in-place, call CSU directly, bypass Policy, unsigned events, artifacts without provenance.  
**Evidence (§8):** Capability = ability not hardware; rules MUST ability/provider/scope/policy; SHOULD cost/evidence; MAY PHM metadata.  
**Evidence (§9):** Subscription + output event schema; MUST causal_refs, failure events, idempotent duplicates; MUST NOT total global order / assume direct delivery.  
**Evidence (§10):** Durable data via Artifacts; MUST NOT mutate; new versions; provenance; validate policies; fail safely.  
**Evidence (§11):** Policy before controlled actions; ALLOW|DENY|REQUIRE; DENY = valid outcome → Event (not system failure).

---

## 5. Standard role contracts (§12–27)

**Evidence (§12):** Roles are **contracts**, not implementations; one binary MAY host multiple roles but MUST expose separate logical contracts.

| Role | § | Purpose (Evidence) | Key MUST NOT / MUST |
|------|---|--------------------|---------------------|
| Context | 13 | PS → Context / Intent | interpret not execute; no physical resources; no final Result; preserve ambiguity |
| Reduction | 14 | Reduce Solution Space | reuse before compute; Evidence on failed lookup; multiple candidates |
| Evidence | 15 | Collect/normalize Evidence | no final truth; no Epistemic Status unless also Epistemic role |
| Epistemic | 16 | Evidence→Status | separate Evidence/Confidence/Scope/Status; no delete old knowledge; contradictions |
| Execution | 17 | Materialize Capsule | only authorized Capsules; no VRA unless also Verification |
| Verification | 18 | Output→VRA | distinguish Output/Result; no silent accept |
| Artifact | 19 | Publish/resolve/supersede | no delete history as normal op |
| Discovery | 20 | Find Capability providers | not nodes; no global registry required |
| Federation | 21 | Membership/import-export | autonomy; no global policy uniformity |
| Settlement | 22 | Receipts/audit | no prompt/private payload; blockchain optional |
| Optimization | 23 | Cost/latency/risk | MUST NOT change Result semantics / override Policy |
| PHM | 24 | Health/risk evidence | evidence not routing decisions |
| Evolution | 25 | Background plane | MUST NOT create Operational Artifact directly |
| Research | 26 | Non-reference exploration | isolated; no direct promote |
| Human Interaction | 27 | Equal normative alternatives | MUST NOT silent collapse (Book 0 A6) |

**Evidence (§37):** Canonical IDs AIRA-CSU-CTX…HUM (15 roles including PHM + HUM).

**Unknown / Tension:** §12 list у тексті закривається на Research; PHM/HUM деталізовані в §24/§27 і §37 — див. TF4-001.

---

## 6. Failure, compatibility, conformance

**Evidence (§28):** Failure → Failure Event; SHOULD Failure Evidence Artifact; typed failures; failure = Evolution evidence.  
**Evidence (§29):** Compatibility = same role contract + ABI/schemas/events/policy/provenance/failures; algorithms may differ.  
**Evidence (§30):** C0 Manifest … C5 Reference (Book I–III tests).  
**Evidence (§31):** Required test classes listed.  
**Evidence (§32):** CSU versioning independent of Core ABI.  
**Evidence (§33–34):** Least privilege; no private data in Events; protected Artifacts + Policy.  
**Evidence (§35–36):** Compliance checklist; exclusions = exact algorithms/UI/backends.  
**Evidence (§38):** Formula CSU = Identity+Manifest+Capability+Event+Artifact+Policy+Lifecycle+Failure.  
**Evidence (§39):** Next = Book IV Reference Implementation (reference only, not protocol requirements).

**Inference:** C0–C5 (Book III) ≠ P0–P3 (Book II) ≠ L0–L3 (Book I) ≠ SCD C0–C5 — TF4-002.
