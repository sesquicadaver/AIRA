# Phase J — Book-gap local remainder (post-I)

**Статус:** складено 2026-08-30; **IN PROGRESS** 2026-08-31 → QUEUE `#199` `#200` **DONE**; first OPEN `#201`.  
**Джерела:** матриця [`implementation-status.md`](implementation-status.md) Book 0–IV після RFC-0078; [`phase-i-plan.md`](phase-i-plan.md) §0.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase J `#199`–`#208`.  
**Резерв RFC:** consolidating RFC-0096 (файл **не** створювати до `#208`).  
**Не змінює:** анти-місію README; production distributed OperationalPlane; Book V operational; C0/C1/C2 merge gates.

## 0. Навіщо (після I, не замість I)

Phase I (`#184`–`#198` / RFC-0078) закрила Handle bind, math.verify, PolicyGate, reuse-index, fail-closed signing, persist/recovery, Clock, envelope replay, UUIDv7 nonce, scoped crypto, MSRV. Ярлик **v0.3-stable**.

Матриця Книг на `main` @ `f274694` лишає локальні **PARTIAL**, які I не брала:

```text
Book I   Opaque Handle — aira_object::object_store_access::mint ще public
Book I   VRA — c1.result.verified_completeness перевіряє schema, не runtime body
Book I   Durable event log — dual-write + plane drain_from memory
Book 0   Evidence B0-005 — schema fixtures, не runtime Claim vs Assumption
Book 0/III Epistemic — CSU є; C1 2+2 не зобов’язаний емітити assessment
Book III Reduction — knowledge catalog unused (reuse-index є)
Book III Verification — text.echo/uppercase presence-only
Book II  9× PARTIAL — local adapters = стеля v0.3, не mesh
Book IV  R2 / stores — Partial C2; plane drain memory objects/events
```

Повна реалізація Книг 0–V **не** є метою J.

```text
Phase I DONE (#198 / RFC-0078 / v0.3-stable)
  → Phase J local Book-gap remainder
    → не Book II wire / не Book V operational
```

## 1. Мета

Закрити залишкові **локальні** MUST Book 0/I/III (і event-store частину Book IV) тестованими атомами. Book II лишається **PARTIAL** з зафіксованою стелею.

Цільовий ярлик після J: **Reference v0.3-strict**.

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; перший OPEN зараз `#201`.
2. Один атом = один PR.
3. C0/C1 не послаблюються; C2 required; C3/C4/C5 не merge gates у J.
4. Core ↛ node/peer/CSU; CSU ↛ CSU.
5. `OperationalPlane` reference-local.
6. Book II не позначати **DONE** лише через local adapter.

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
Human Interaction CSU (Final Collapse лишається STUB)
HTTP/peer POST-MVP productization
making cargo-deny / C3 a merge gate
```

## 4. Смуги (одна лінія)

```text
J0 govern + Book II ceiling honesty
  → J1 Book I remainder (Handle API, VRA payload, event-log authority)
    → J2 Book 0/III remainder (catalog, text.verify, evidence runtime, epistemic emit)
      → J3 docs + consolidating RFC-0096
```

## 5. Атоми → QUEUE `#199`–`#208`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#199` | J0 | Phase J wiring | цей план **IN PROGRESS**; QUEUE `#199` DONE + `#200`–`#208` OPEN; `phase_j_doc` | Book II honesty (#200) |
| `#200` | J0 | Book II ceiling honesty | матриця envelope/EP/AP/identity/discovery/CAP/CRP/settlement: **PARTIAL** + «local adapter = v0.3 ceiling»; `phase_j_doc` | Handle API (#201) |
| `#201` | J1 | Seal `object_store_access` | `mint` не в public prelude `aira-object` для CSU; store-backend лише `aira-core`; тест | VRA payload (#202) |
| `#202` | J1 | VRA runtime B1-010 | C1 `calculate_2_plus_2` artifact body містить усі `required` з `schemas/result/verified-result-artifact.schema.json` | event-log (#203) |
| `#203` | J1 | Event-log authority | після persist reopen `event_tail` читає `events/file-chain-log.json`; memory `drain_from` не єдине джерело | catalog (#204) |
| `#204` | J2 | Reduction catalog bind | plane Reduction бачить durable reuse без ручного `enable_ready_solution`; тест | text.verify (#205) |
| `#205` | J2 | Semantic verify text.* | `text.echo` / `text.uppercase`: wrong string → not VERIFIED | evidence (#206) |
| `#206` | J2 | Evidence primacy runtime | Claim vs Assumption reject у runtime (не лише fixtures `#125`) | epistemic (#207) |
| `#207` | J2 | Epistemic emit on C1 | C1 2+2 path пише epistemic-assessment artifact (не full Epistemic plane) | RFC (#208) |
| `#208` | J3 | Phase J docs + RFC-0096 | README/status **v0.3-strict**; RFC-0096; QUEUE J closed | — |

```text
#199–#200 J0
  → #201–#203 J1
    → #204–#207 J2
      → #208 J3 docs
```

Human Final Collapse (**STUB**) і Goal Compiler / Evolution (**RESEARCH**) у J **не** входять.

## 6. Acceptance Phase J

```text
QUEUE `#199`–`#208` DONE; no OPEN J atoms.
Book II protocol rows still PARTIAL (ceiling tested).
Handle mint not a public CSU prelude API.
C1 VRA body matches schema required[].
Event tail survives reopen via file-chain.
Reuse catalog path without manual plane pre-seed.
text.echo/uppercase wrong value not VERIFIED.
Claim/Assumption enforced at runtime.
C1 path emits epistemic assessment.
Анти-місія не порушена; C3/C4/C5 не merge gates.
```

## 7. Мапінг матриця → атоми

| Book row (status @ f274694) | Evidence in tree | Atom |
|-----------------------------|------------------|------|
| I Opaque Handle PARTIAL | `pub use handle::object_store_access` у `crates/aira-object/src/lib.rs` | `#201` |
| I VRA completeness PARTIAL | `c1.rs` schema-required vs runtime body | `#202` |
| I Durable event log PARTIAL | `plane.rs` `drain_from`; `local.rs` file-chain dual-write | `#203` |
| III Reduction PARTIAL | `enable_ready_solution`; catalog unused | `#204` |
| III Verification PARTIAL | `verification-basic` `text.echo` presence-only | `#205` |
| 0 Evidence PARTIAL | fixtures `#125`; не runtime | `#206` |
| 0/III Epistemic PARTIAL | `epistemic_assessment_roundtrip_*`; C1 2+2 optional | `#207` |
| II ×9 PARTIAL | local adapters H/G | `#200` |
| IV R2 / stores PARTIAL | C2 local; plane memory drain | `#200` `#203` |
| — | wiring / closure | `#199` `#208` |

## 8. RFC

- Consolidating (`#208`): `specs/rfc/AIRA-RFC-0096-phase-j-book-gap-local.md` — id **confirmed free** (немає `AIRA-RFC-0096*` у `specs/rfc/`).
- Per-atom RFC при ABI Handle (`#201`), VRA body (`#202`), Verification text (`#205`).

## 9. Activation rule

```text
#198 DONE (RFC-0078) → цей план активовано атомом `#199`.
#199 wiring DONE; `#200` Book II ceiling honesty DONE; first OPEN `#201` (Seal `object_store_access`).
RFC-0096 файл лише в `#208`.
```
