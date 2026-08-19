# Book IV — Reference Implementation (аналіз)

**Джерело:** `Manifesto etc/Book IV — Reference Implementation v0.1.md`  
**Type:** Reference / Non-Normative  
**Depends on:** Book 0–III  
**Формат:** structured digest §1–31 після повного читання (1319 рядків)  
**Confidence:** High (Evidence); Medium (Inference)

---

## 1. Роль

**Evidence (§0–2):** Не новий стандарт; доводить мінімальну локальну реалізацію без ML/GPU/Blockchain/Federation/PHM/Routing Optimizer/Distributed Execution/Research Plane.  
**Evidence (§1):** Мінімальний потік PS→Context→Reduction→Execution→Verification→VRA.  
**Evidence (§2):** Не production distributed / cloud / model hosting / GPU scheduler / blockchain / agent framework / research optimizer.

**Inference:** Book IV = executable proof of Books 0–III, не джерело сумісності саме по собі (SCD: Standard = 0–III + Conformance).

---

## 2. Architecture, language, process, storage

**Evidence (§3):** Tree `aira-node` = core-runtime + protocols + csu + storage + api + tests.  
**Evidence (§4):** SHOULD Rust; Go/TS/Python prototype OK; Core Rust recommended.  
**Evidence (§5):** Single process OK; logical isolation; CSU only via Core ABI abstraction.  
**Evidence (§6):** Object Store (SQLite recommended); Artifact Store content-addressed FS; Event Log append-only SQLite with causal refs / idempotent insert / signatures.

---

## 3. Core runtime pieces

**Evidence (§7):** Object Runtime create/validate/hash/store/handle; JSON descriptor example; Handle `aira:<type>:<id>`.  
**Evidence (§8):** Local pub/sub + persist; no global total ordering.  
**Evidence (§9):** Policy ALLOW|DENY|REQUIRE; deny-by-default private; never domain logic.  
**Evidence (§10):** Invariant list matches Book I spirit; InvariantViolation event.  
**Evidence (§11–12):** CSU packaging native/WASM/in-process/IPC; v0.1 recommend in-process Rust trait; `trait Csu` is implementation detail — normative remains Book III.

---

## 4. Minimal reference CSU set (§13)

**Evidence:** Context (rule-based, no ML); Reduction (local ready/knowledge lookup); Execution (echo / shell-disabled / deterministic; no arbitrary shell); Verification (schema/hash/expected/policy/manual); Artifact (CAS+SQLite+SHA-256+Ed25519); Evidence; Epistemic (simple rule mapping).

**Inference:** Subset of Book III §37 — no full Discovery/Fed/Settlement/PHM/Evolution/Research/Human as first-class reference CSU (stubs in §14 for some protocols).

---

## 5. Local protocols & operational flow

**Evidence (§14):** Local Event/Artifact/Identity; Discovery=local registry; CRP=single-node binding; Federation stub (not R0); Settlement=local signed receipt, no blockchain.  
**Evidence (§15):** Submit→Interpret→Reduce→Materialize→Verify→Evolve (Evidence/Epistemic).

---

## 6. API, CLI, config, security, formats

**Evidence (§16):** HTTP `/v1/problems|results|artifacts|events|capabilities|csu`.  
**Evidence (§17):** CLI init/identity/csu/problem/result/artifact/event/conformance; demo `Calculate 2 + 2`.  
**Evidence (§18):** YAML config local mode, sqlite/fs, sandbox, autoload basic CSUs.  
**Evidence (§19):** Network/shell off; scoped FS; unsigned rejected.  
**Evidence (§20):** JSON v0.1; canonical JSON→SHA-256; Ed25519.

---

## 7. Testing, harness, profiles, MVP, layout

**Evidence (§21):** Unit/integration/conformance groups.  
**Evidence (§22):** `aira conformance run --profile …`; harness profile names R0–R4 (minimal-local … federation-ready).  
**Evidence (§23):** Reference Profiles R0 Minimal Local Core … R4 Research (R4 not required).  
**Evidence (§24):** MVP = ontology+local core+local protocols+basic CSU+CLI+store+harness; prove PS→…→Evidence.  
**Evidence (§25–26):** Suggested `aira/` crates layout + crate list.  
**Evidence (§27):** MUST NOT add ontology / bypass ABI / mutate / hide failures / require ML|GPU|network|blockchain|federation|distributed. MAY simple algorithms/stubs/optional modes.  
**Evidence (§28):** Speculative → Book V.  
**Evidence (§29–30):** Deliverables + success criteria.  
**Evidence (§31):** Claim: local deterministic event-native artifact-based CSU system without ML/GPU/blockchain/federation/distributed; next Book V.

**Tension:** §22 harness profile labels ≠ §23 profile descriptions naming — TF5-001.
