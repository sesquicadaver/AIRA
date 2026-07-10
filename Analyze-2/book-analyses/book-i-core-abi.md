# Book I — Core Architecture & ABI (аналіз)

**Джерело (immutable):** `Manifesto etc/Book I — Core Architecture & ABI v0.1.md`  
**Depends on:** Book 0 (`Manifesto etc/Book 0 — …`)  
**Цикл:** Analyze-2  
**Формат:** structured digest усіх §1–25 після **повного** читання файлу (не вибірковий фрагмент).  
**Confidence:** High для фактів з Evidence; Medium для Inference.

---

## 1. Роль

**Evidence (§0 status + §24 + §25):** Type Normative; Scope AIRA Core; Depends on Book 0; Exports Object Model, Stable ABI, Event Runtime, Policy Gate, Invariant Checker, Security Boundary, CSU Runtime Contract.

**Evidence (§24):**

```text
AIRA Core = Object Model + Stable ABI + Event Runtime
          + Policy Gate + Invariant Checker
          + Security Boundary + CSU Runtime
```

**Inference:** Book I операціоналізує межу ядра Book 0 §11 у сумісний ABI-контракт.

---

## 2. Core Mission & Non-Goals

**Evidence (§1):** Core не вирішує задачі; гарантує object model, ABI, events, policy, invariants, security, isolated CSU loading; середовище PS→VRA через CSU.

**Evidence (§2):** MUST NOT: LLM/GPU/scheduler/routing/ML/prediction/PHM/blockchain/knowledge algorithms/optimization/domain-specific logic.

**Inference:** Пряме продовження Book 0 §11 Non-kernel list.

---

## 3. Core Boundary & Object Model

**Evidence (§3):** Підсистеми Core = Object Model, Stable ABI, Event Runtime, Policy Gate, Invariant Checker, Security Boundary, CSU Runtime.

**Evidence (§4):** Канонічні об’єкти = список Book 0 (PS, Context, Evidence, Epistemic Status, Execution Intent, Capsule, Capability, Artifact, Event, Policy, CSU, VRA). GPU/LLM/Node/Driver/… не fundamental.

**Evidence (§5):** Мінімальний descriptor (object_id…signature); objects MUST be immutable; зміна → новий object або event.

---

## 4. Handle Model

**Evidence (§6):** Список handles (ProblemHandle…ResultHandle); opaque; CSU MUST NOT assume internal structure; no raw pointers to core structs.

---

## 5. Stable ABI

**Evidence (§7):** Мінімальний ABI: object_open/read/emit/subscribe, policy_check, capability_register, artifact_publish/resolve, csu_register/suspend/resume/unload.  
ABI MUST NOT містити GPU/model/prompt/wallet/route/scheduler/blockchain.

---

## 6. Event Runtime & Classes

**Evidence (§8):** CSU MUST NOT call each other directly; emit → Core validate → observe. Event descriptor з causal_refs; MUST support causality; MUST NOT guarantee global sync state.

**Evidence (§9):** Базові класи: ObjectCreated, ObjectReferenced, ArtifactPublished, CapabilityRegistered, PolicyEvaluated, CapsuleCreated/Bound/Completed, VerificationCompleted, ResultPublished, CSURegistered/Suspended/Failed, InvariantViolation.

---

## 7. Policy Gate

**Evidence (§10):** Повертає лише ALLOW | DENY | REQUIRE; не виконує дій; Policy Query (subject/action/object/context/evidence/time); операції з Policy — через Gate.

---

## 8. Invariant Checker

**Evidence (§11):** Мінімум: object immutability; artifact provenance; event signature; policy enforcement; CSU isolation; no direct CSU-to-CSU mutation; no Core semantic leakage; VRA completeness; Operational/Evolution separation.  
MUST stop violating ops; MUST NOT optimize.

---

## 9. Security Boundary

**Evidence (§12):** CSU isolated; MUST NOT access core memory, foreign CSU state, private payloads, user data, policy internals, raw execution without permission. Access via ABI+Policy. SHOULD: sandbox, capabilities, signed manifests, integrity, event sigs, revocation, least privilege.

---

## 10. CSU Runtime & Lifecycle

**Evidence (§13):** CSU = only extension form; examples Context/Execution/Evidence/Verification/Artifact/Discovery/Settlement/PHM/Optimization/Research; Core не знає внутрішньої реалізації; manifest fields; **Driver → CSU** (Driver лише можливий low-level subtype).

**Evidence (§14):** Discovered→Registered→Verified→Active→Suspended→Revoked→Archived (+ підсекції 14.1–14.6).

---

## 11. Capability, Artifact, VRA

**Evidence (§15):** Capability = можливість зменшити невизначеність / матеріалізувати частину рішення; Core не інтерпретує зміст.

**Evidence (§16):** Artifact content-addressed або equivalent integrity; єдиний механізм для всіх типів Artifact; Core не знає семантики.

**Evidence (§17):** Core не створює VRA сам; гарантує контракт; мінімальний Result Descriptor; VRA = штатне завершення Operational Pipeline.

---

## 12. Op/Evo, Failure, Compatibility, Versioning

**Evidence (§18):** Evolution path Evidence→Validation→Promotion→Operational Use; Evolution CSU не створює Operational Artifact без Promotion.

**Evidence (§19):** Failure MUST be Event or Artifact; мінімальні failure events listed; failure → Evidence for Evolution.

**Evidence (§20):** Compatibility = same object model, compatible ABI, same invariant checks, compatible event/artifact descriptors, compatible VRA.

**Evidence (§21):** Незалежні версії Core ABI / Object / Event / Artifact / Policy / CSU Manifest; algorithm/CSU/protocol changes не повинні ламати ABI якщо виразні через існуючі контракти.

---

## 13. Conformance Levels

**Evidence (§22):** L0 Minimal Core; L1 + local CSU/Artifact/VRA; L2 + Book II; L3 Book I–III compatible.

**Inference / Unknown:** Відповідність SCD AIRA-C0…C5 не 1:1 за назвами — див. TF2-001.

---

## 14. Forbidden practices & Status

**Evidence (§23):** MUST NOT: domain logic in Core; CSU-to-CSU outside ABI; in-place Artifact mutate; Policy bypass; hide failure; change object model without version bump; ML in Kernel; depend on specific GPU/LLM/blockchain.

**Evidence (§25):** Next = Book II protocols (Discovery, Artifact, Event, Federation, Identity, Settlement, CRP); Book II MUST NOT describe algorithms.
