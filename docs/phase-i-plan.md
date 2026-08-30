# Phase I — Semantic contract stabilization (post-H)

**Статус:** складено 2026-08-28; **IN PROGRESS** 2026-08-30 → QUEUE `#184`–`#185` **DONE**; first OPEN `#186`.  
**Джерела:** незалежний аудит `main` @ `b66bcf1` (Handle / Verification / Policy / Reuse / identity / persistence); [`implementation-status.md`](implementation-status.md); [`phase-h-plan.md`](phase-h-plan.md) §7.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase I `#184`–`#198` (після `#183` DONE).  
**Не змінює:** анти-місію README; Book semantics без per-atom RFC; production distributed OperationalPlane.

## 0. Навіщо Phase I (після H, не замість H)

Phase H (`#152`–`#183`) правильно нарощує protocol depth (C3 → CRP → settlement → promotion → v0.3).

Аудит показав, що **кілька фундаментальних контрактів** нижче протокольного шару ще мають неправильну або неповну семантику:

```text
Handle integrity
CSU effect-level Policy Gate
Semantic Verification (VERIFIED ≠ correct)
Reuse before compute (LocalSession path)
Identity fail-closed (no local-test fallback)
Crash-safe / recoverable persistence
```

CRP/settlement на некоректних базових контрактах лише закріпить хибну семантику. Тому Phase I — **stabilization / semantic strictness** уже реалізованих механізмів, а не новий protocol ladder і не вставка всередину `#163`–`#183`.

```text
Phase H DONE (#183 / RFC-0077 / Reference v0.3)
  → Phase I semantic contracts
    → лише потім новий protocol/product plan (якщо потрібен)
```

## 1. Мета

Підняти вже реалізовані Core/CSU/flow/crypto/persistence контракти до тієї ж якості, що вже мають canonical signatures, event equivocation і peer trust reject `local-test`.

Цільовий ярлик після I: **Reference v0.3-stable** (або v0.3.1 у docs) — без анти-місії, без distributed plane.

## 2. Інваріанти

1. **Лінійність:** `#N` стартує лише після `#N-1` = DONE; перший атом I — лише після `#183` DONE.
2. **Один атом = один PR.**
3. Не вставляти атоми Phase I між `#163` і `#183`.
4. C0/C1 gate не послаблюється; C2 required job лишається.
5. Core ↛ node/peer/CSU; CSU ↛ CSU — firewall.
6. `OperationalPlane` лишається reference-local.
7. Status-матриця має бути **чесною**: якщо тест доводить лише Debug opacity — не писати Opaque Handle = DONE.

## 3. Анти-місія (ніколи в QUEUE / «Phase I»)

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
hostile-process / WASM CSU sandbox
production secrets HSM
App Store / enterprise Authenticode beyond helpers
```

## 4. Смуги (одна лінія)

```text
I0 govern + status honesty
  → I1 P0 Core/CSU semantics (Handle, Verify, Policy, Reuse)
    → I2 P1 identity + durability
      → I3 P2 hardening (replay, nonce, crypto scope, CI)
        → I4 docs + consolidating RFC
```

## 5. Атоми → QUEUE (після `#183`; номери `#184+`)

| ID | Підфаза | Атом | Done when |
|----|---------|------|-----------|
| `#184` | I0 | Phase I wiring | `docs/README` + QUEUE + phase-h §7 → цей план; living/contract smoke ids `#184`–хвіст |
| `#185` | I0 | Status honesty rollup | `implementation-status`: Opaque Handle, Reduction/reuse, Verification — **PARTIAL** (або точніше); notes з audit refs |
| `#186` | I1 | Handle integrity | `Handle::new` / `storage_token` не публічні для чужого crate; `open` bind `object_id == handle.object_ref`; adversarial tests (forged/cross-object/cross-store) |
| `#187` | I1 | Semantic verify math.eval.safe | VerificationBasic незалежно обчислює expression з capsule/output і порівнює; wrong finite number → not VERIFIED; тест |
| `#188` | I1 | CSU PolicyGate in invoke | `CsuExecutionContext` отримує bound gate з runtime; `check_policy` працює; fail-closed без gate; тест |
| `#189` | I1 | Durable reuse index | LocalSession/plane path: повторний той самий problem → ready solution без повторного execution; persistent index; e2e тест (не лише manual pre-seed) |
| `#190` | I2 | Fail-closed signing | `active_signature` без fallback на `local-test` у normal node; demo/test mode явний; LocalSession не ковтає identity errors мовчки |
| `#191` | I2 | Atomic session persist | `persist_after_submit`: temp+rename (або еквівалент) для problems/events metadata; corrupt problems index ≠ silent empty wipe |
| `#192` | I2 | Artifact metadata recovery | descriptor per artifact-id (або SQLite metadata); shared content hash не знищує другий descriptor; recovery/doc/test |
| `#193` | I2 | Runtime Clock | `Clock` trait; SystemClock у runtime; FixedClock у тестах; operational artifacts не всі на `2026-07-10T12:00:00Z` |
| `#194` | I3 | Envelope freshness/replay | `expires_at` / skew / message-id replay window на peer receive (policy window); тести |
| `#195` | I3 | Run nonce concurrency | UUIDv7 або transactional sequence замість racy `run-counter` parse/write; тест concurrent/process |
| `#196` | I3 | Instance-scoped crypto | прибрати обов’язковий process-global OnceLock signer для embed/tests або document+API для scoped Keyring; мінімальний тест ізоляції |
| `#197` | I3 | MSRV + supply-chain CI | узгодити `rust-version` із CI **або** MSRV job; pin Actions за commit SHA **або** documented accept; `cargo deny`/`audit` job (non-blocking спочатку OK) |
| `#198` | I4 | Phase I docs + RFC | README + implementation-status stable tag; consolidating RFC (напр. RFC-0078); QUEUE I closed |

```text
#184–#185 I0
  → #186–#189 I1 P0
    → #190–#193 I2 P1
      → #194–#197 I3 P2
        → #198 I4 docs
```

## 6. Acceptance Phase I

```text
Handle: не forgeable з-поза store crate; open bind-checked; adversarial tests green.
Verification: wrong math result не VERIFIED.
CSU invoke: PolicyGate bound; effect-level check_policy usable.
Reuse: LocalSession повтор submit → reuse path (durable index).
Signing: no silent local-test fallback outside explicit demo/test.
Persistence: no silent wipe of corrupt problems index; atomic write path для metadata.
Artifact: multi-artifact same content-hash recoverable without sole reliance on one fragile index.json semantics (per plan atom).
Clock: runtime ≠ forever fixed MVP timestamp.
CI: MSRV claim evidence або claim removed; supply-chain note/job.
Docs/RFC: Phase I closed; no OPEN I atoms.
Анти-місія не порушена.
```

## 7. Мапінг audit → атоми

| Audit priority | Problem | Atom |
|----------------|---------|------|
| P0 | Opaque Handle integrity | `#186` |
| P0 | Semantic Verification | `#187` |
| P0 | CSU action Policy Gate | `#188` |
| P0 | Durable reuse index | `#189` |
| P1 | Remove local-test fallback | `#190` |
| P1 | Atomic metadata persistence | `#191` |
| P1 | Artifact index/descriptor recovery | `#192` |
| P1 | Runtime clock | `#193` |
| P2 | Replay/expiry | `#194` |
| P2 | run-counter concurrency | `#195` |
| P2 | instance-scoped crypto | `#196` |
| P2 | MSRV + supply-chain CI | `#197` |
| — | Status honesty / wiring / closure | `#184` `#185` `#198` |

## 8. RFC

- Consolidating: `specs/rfc/AIRA-RFC-0078-phase-i-semantic-stabilization.md` (`#198`) — id **confirmed free** at `#184` wiring (no `AIRA-RFC-0078*` in `specs/rfc/`).
- Per-atom RFC при ABI-зміні Handle, Verification contract, Clock у public API.

## 9. Activation rule

```text
#183 DONE (RFC-0077) → цей план став активною чергою.
#184 wiring DONE; #185 status honesty DONE.
first OPEN `#186` (Handle integrity).
```
