# Phase K — Local LLM Execution CSU (post-J, post-D)

**Статус:** складено 2026-09-01; **DONE** 2026-09-01 → QUEUE `#209`–`#216` (RFC-0104). QUEUE K closed.  
**Джерела:** рішення розробника 2026-09-01 (Desktop Work лишається C1 `execution-basic`; inventory Phase D **DONE**; Book I §2 LLM Backend = зовнішній CSU); [`EVO-3.md`](../EVO-3.md) §8–§10; [`phase-d-plan.md`](phase-d-plan.md) (D7 DONE; execution bind не входив у D); [`phase-j-plan.md`](phase-j-plan.md) QUEUE J closed.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase K `#209`–`#216`.  
**Closure RFC:** [`AIRA-RFC-0104`](../specs/rfc/AIRA-RFC-0104-phase-k-local-llm-csu.md) — id **confirmed free** at `#209`.  
**Не змінює:** анти-місію README (ядро **не** хост інференсу; **не** GPU marketplace); C0/C1 `Calculate 2 + 2`; `aira-core`; canonical `llm_model_id`.

## 0. Навіщо (після J і D, не замість них)

Phase D дала модель як Artifact + Policy + activate **без виконання**.  
C1 plane виконує лише `math.eval.safe` / `text.echo` / `text.uppercase`.  
Book I: LLM Backend — **CSU**, не Core. EVO-3 §10: inventory check → local execution candidate; модель не початкова точка.

```text
Phase D inventory/activate DONE (#53–#74)
  → Phase J reference-strict DONE (#199–#208)
    → Phase K: bind activated local model to Execution CSU
```

## 1. Мета

Problem Statement, який не є безпечною арифметикою/echo, може завершитись Verified Result Artifact через **Execution CSU**, що викликає **зовнішній** локальний backend (mock у тестах; процес ollama/llama.cpp — окремий атом), використовуючи **активовану** модель з Phase D.

Цільовий ярлик: **Reference v0.3-strict + local LLM CSU** (не v0.4 protocol; не marketplace).

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` не містить LLM logic / tokenizer / ggml.
3. CSU ↛ CSU: `execution-llm` не залежить Cargo від `model-inventory` / `model-acquisition`; plane/CLI оркеструє.
4. Activation лишається окремою від execution (D4 `#64`).
5. C1 `Calculate 2 + 2` лишається `execution-basic` + VERIFIED 4.0.
6. Немає backend / немає activate → **CapsuleFailed** + Evidence, не фальшивий VERIFIED.
7. Download/marketplace/remote model fetch — **Out**.
8. Не редагувати `Manifesto etc/`, `Meditation_About/`.

## 3. Анти-місія (ніколи в QUEUE як «зробити AIRA LLM runtime»)

```text
GPU marketplace / compute rental
LLM runtime (Core як inference host)
blockchain network / scheduler in Core
Core що «вирішує задачі» замість CSU contracts
```

Phase K **є** дозволеним: LLM Backend CSU + Artifact weights + Capability. Це не анти-місія.

## 3.1. Поза цим етапом

```text
full Book II wire / distributed OperationalPlane
model marketplace / leaderboard / paid GPU
WASM/hostile sandbox for weights
changing Book 0 canonical pipeline (RFC-A)
verification of free-form LLM output beyond fail-closed provenance (full semantic LLM verify)
```

## 4. Смуги (одна лінія)

```text
K0 govern (wiring)
  → K1 capsule contract
    → K2 execution-llm CSU + mock backend
      → K3 Reduction action bind (2+2 unchanged)
        → K4 plane register CSU
          → K5 activate gate
            → K6 process backend (fail-closed if binary missing)
              → K7 Desktop/docs + RFC-0104
```

## 5. Атоми → QUEUE `#209`–`#216`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#209` | K0 | Phase K wiring | цей план **IN PROGRESS**; QUEUE `#209` DONE + `#210`–`#216` OPEN; `phase_k_doc` | capsule schema (#210) |
| `#210` | K1 | Capsule `text.generate.local` | payload schema `aira:schema:execution:generate-local:0.1` + fixtures; RFC-0105 **DONE** | CSU crate (#211) |
| `#211` | K2 | `execution-llm` CSU + mock | CapsuleCreated → CapsuleCompleted via `MockBackend`; без backend → fail-closed; не в `aira-core` **DONE** | Reduction (#212); ollama (#215) |
| `#212` | K3 | Reduction bind | non-math/echo/upper → `text.generate.local`; `Calculate 2 + 2` лишається `math.eval.safe` **DONE** (RFC-0107) | plane register (#213) |
| `#213` | K4 | Plane register | `OperationalPlane`/`LocalSession` диспатчить generate на `execution-llm`; mock completes in tests **DONE** (RFC-0108) | activate gate (#214) |
| `#214` | K5 | Activate gate | generate без Phase D activate → CapsuleFailed + Evidence, не VERIFIED **DONE** (RFC-0109) | process backend (#215) |
| `#215` | K6 | Process backend | argv/local CLI adapter; missing binary fail-closed; тести лишають mock **DONE** (RFC-0110) | Desktop RFC (#216) |
| `#216` | K7 | Desktop + RFC-0104 | Work tab generate path; consolidating RFC-0104; QUEUE K closed **DONE** | — |

```text
#209 K0
  → #210 K1 schema
    → #211 K2 CSU mock
      → #212–#213 K3–K4 bind
        → #214 K5 activate
          → #215 K6 process
            → #216 K7 close
```

## 6. Acceptance Phase K

```text
QUEUE `#209`–`#216` DONE; no OPEN K atoms.
C1 2+2 still VERIFIED 4.0 via execution-basic.
Non-math problem can complete via execution-llm (mock in CI).
No activated model → fail-closed, not fake VERIFIED.
aira-core has no inference.
Анти-місія (marketplace / Core-host) не порушена.
```

## 7. RFC

- Consolidating (`#216`): [`AIRA-RFC-0104`](../specs/rfc/AIRA-RFC-0104-phase-k-local-llm-csu.md) — id **confirmed free** at `#209`.
- Per-atom RFC-S/RFC-D/RFC-E у `#210`–`#215` (окремі файли; не RFC-A).

## 8. Activation rule

```text
#208 DONE (RFC-0096) + рішення розробника 2026-09-01 → цей план активовано атомом `#209`.
#209 wiring DONE; `#210` generate-local schema DONE (RFC-0105); `#211` execution-llm mock DONE (RFC-0106); `#212` Reduction bind DONE (RFC-0107); `#213` Plane register DONE (RFC-0108); `#214` Activate gate DONE (RFC-0109); `#215` Process backend DONE (RFC-0110); `#216` Desktop + RFC-0104 DONE.
QUEUE K closed.
```
