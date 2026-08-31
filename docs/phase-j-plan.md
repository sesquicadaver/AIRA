# Phase J — Book-gap local remainder (post-I)

**Статус:** складено 2026-08-30; **PLANNED** (не активовано). не в QUEUE як рядки `#199+`. У [`QUEUE.md`](../QUEUE.md) **немає** OPEN.  
**Джерела:** матриця [`implementation-status.md`](implementation-status.md) Book 0–IV (**PARTIAL** після RFC-0078); [`phase-i-plan.md`](phase-i-plan.md) §0 («лише потім новий protocol/product plan»).  
**Канон backlog:** досі [`QUEUE.md`](../QUEUE.md). Цей файл — **план-перед-чергою**, не черга.  
**Резерв id:** QUEUE `#199`–`#208`; consolidating RFC-0096 (файл **не** створювати до wiring).  
**Не змінює:** анти-місію README; production distributed OperationalPlane; Book V operational; C0/C1/C2 merge gates.

## 0. Навіщо (після I, не замість I)

Phase I (`#184`–`#198` / RFC-0078) закрила семантичні діри Handle/Verify/Policy/Reuse/identity/persist і поставила ярлик **v0.3-stable**. Матриця Книг лишає **PARTIAL**, які I свідомо не закривала:

```text
Book I   Opaque Handle (store-access ще public API)
Book I   VRA runtime payload vs B1-010 required fields
Book I   Durable event log (dual-write / plane memory drain)
Book 0/III Evidence primacy, Epistemic coordinate, text.verify, knowledge catalog
Book II  9× PARTIAL = local adapters (стеля v0.3, не mesh)
Book IV  R2 / stores PARTIAL
```

Повна реалізація Книг 0–V **не** є метою J. Book II network wire, distributed plane, Book V promotion-to-operational, GPU/LLM/blockchain/scheduler — **поза етапом**.

```text
Phase I DONE (#198 / RFC-0078 / v0.3-stable)
  → цей план PLANNED
    → QUEUE J лише після окремого wiring-атома (як #184 після #183)
```

## 1. Мета

Підняти залишкові **локальні** MUST з матриці Book 0/I/III/IV до DONE (або чесний ABSENT/PARTIAL з тестом), не змінюючи стелю Book II = in-process reference.

Цільовий ярлик після J (лише якщо активовано й закрито): **Reference v0.3-strict** — без анти-місії, без mesh.

## 2. Інваріанти

1. **Не в QUEUE**, доки немає wiring-атома після явного рішення розробника.
2. Один атом = один PR; лінійність `#N` після `#N-1`.
3. C0/C1 не послаблюються; C2 required job лишається; C3/C4/C5 не стають merge gates у J.
4. Core ↛ node/peer/CSU; CSU ↛ CSU.
5. `OperationalPlane` reference-local.
6. Book II рядки не позначати **DONE** лише тому, що є local adapter (стеля = PARTIAL, атом honesty).

## 3. Анти-місія (ніколи в QUEUE / «Phase J»)

```text
GPU marketplace / compute rental
LLM runtime (Core як inference host)
blockchain network / blockchain-required settlement
scheduler / distributed job orchestration (Core)
Core що «вирішує задачі» замість CSU contracts
```

## 3.1. Поза цим етапом

```text
full Book II wire / discv5 production mesh
distributed OperationalPlane
Book V item promoted to operational
hostile-process / WASM CSU sandbox
Human Interaction CSU (Final Collapse лишається STUB, якщо не окремий атом поза J)
HTTP/peer POST-MVP productization
making cargo-deny / C3 a merge gate
```

## 4. Смуги (одна лінія)

```text
J0 govern + Book II ceiling honesty
  → J1 Book I remainder (Handle API, VRA payload, event-log authority)
    → J2 Book 0/III remainder (reuse catalog, text.verify, evidence runtime, epistemic emit)
      → J3 docs + consolidating RFC-0096
```

## 5. Атоми → майбутня QUEUE (номери резерв; **не** рядки QUEUE зараз)

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#199` | J0 | Phase J wiring | `docs/README` + QUEUE рядки `#199`–`#208` OPEN-хвіст + цей план **IN PROGRESS**; `phase_j_doc` | status honesty (#200) |
| `#200` | J0 | Book II ceiling honesty | матриця: envelope/EP/AP/identity/discovery/CAP/CRP/settlement лишаються **PARTIAL** з нотаткою «local adapter = v0.3 ceiling»; тест `phase_j_doc` | Handle API (#201) |
| `#201` | J1 | Seal `object_store_access` | `mint` недоступний з depending crate без store-backend (не `pub use` на корені `aira-object` для CSU); adversarial/compile-fail або crate-graph тест | VRA payload (#202) |
| `#202` | J1 | VRA runtime B1-010 | C1 `calculate_2_plus_2` artifact body містить усі `required` з `verified-result-artifact.schema.json` (не лише schema-case `#126`) | event-log (#203) |
| `#203` | J1 | Event-log authority | після persist `event_tail` / reopen читає `file-chain-log.json` як джерело правди; memory drain не є єдиним шляхом | reuse catalog (#204) |
| `#204` | J2 | Reduction catalog bind | plane Reduction бачить durable reuse без ручного `enable_ready_solution` на in-memory catalog; тест | text.verify (#205) |
| `#205` | J2 | Semantic verify text.* | `text.echo` / `text.uppercase`: wrong string → not VERIFIED (не `result` presence-only) | evidence (#206) |
| `#206` | J2 | Evidence primacy runtime | Claim vs Assumption: runtime reject (не лише `schema validate` fixtures B0-005) | epistemic (#207) |
| `#207` | J2 | Epistemic emit on C1 | C1 2+2 path пише epistemic-assessment artifact (EPI-001 plane path, не full Epistemic plane) | RFC (#208) |
| `#208` | J3 | Phase J docs + RFC-0096 | README/status tag; consolidating RFC-0096; QUEUE J closed | — |

```text
#199–#200 J0
  → #201–#203 J1
    → #204–#207 J2
      → #208 J3 docs
```

Human Final Collapse (**STUB**, Book 0 A6) і Goal Compiler / Evolution (**RESEARCH**) у J **не** входять.

## 6. Acceptance Phase J (лише після активації й close)

```text
QUEUE має `#199`–`#208` DONE; no OPEN J atoms.
Book II protocol rows still PARTIAL (ceiling documented + tested).
Handle mint not a public CSU/object prelude API.
C1 VRA body matches schema required[].
Event tail survives reopen via file-chain as authority.
Reuse catalog path without manual plane pre-seed.
text.echo/uppercase wrong value not VERIFIED.
Claim/Assumption enforced at runtime.
C1 path emits epistemic assessment.
Анти-місія не порушена; C3/C4/C5 не merge gates.
```

## 7. Мапінг матриця → атоми

| Book row (status зараз) | Problem | Atom |
|-------------------------|---------|------|
| I Opaque Handle PARTIAL | `object_store_access::mint` з depending crate | `#201` |
| I VRA completeness PARTIAL | runtime payload minimal vs schema required | `#202` |
| I Durable event log PARTIAL | dual-write / memory drain | `#203` |
| III Reduction PARTIAL | knowledge catalog unused | `#204` |
| III Verification PARTIAL | text.* presence-only | `#205` |
| 0 Evidence PARTIAL | B0-005 schema-only | `#206` |
| 0/III Epistemic PARTIAL | full plane out; emit missing on C1 | `#207` |
| II ×9 PARTIAL | не піднімати до DONE без mesh | `#200` |
| IV R2 / stores PARTIAL | частково покрито `#203`; решта стеля | `#200` + `#203` |
| — | wiring / closure | `#199` `#208` |

## 8. RFC

- Consolidating (після close): `specs/rfc/AIRA-RFC-0096-phase-j-book-gap-local.md` — id **confirmed free** 2026-08-30 (немає `AIRA-RFC-0096*` у `specs/rfc/`). Файл лише в атомі `#208`.
- Per-atom RFC обов’язковий при ABI Handle (`#201`), VRA body (`#202`), Verification text contract (`#205`).

## 9. Activation rule

```text
Цей план = PLANNED.
#198 DONE (RFC-0078) ≠ автоактивація J.
Активація = окремий атом `#199` (як `#184` після `#183`): рядки QUEUE + phase_j_doc IN PROGRESS.
До `#199` у QUEUE немає `| 199 |` / OPEN-хвоста J.
```
