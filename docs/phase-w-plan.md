# Phase W — Pack 1 residual honesty (post-V audit)

**Статус:** **IN PROGRESS** (активовано 2026-09-11; `#331` wiring; first OPEN `#332`).  
**Джерела:** пост-V аудит `p-only/AIRA-audit-d1115f2-2026-09-11.md` @ `d1115f2` (merge PR #295 / RFC-0208); Phase V **DONE** @ [`phase-v-plan.md`](phase-v-plan.md) / RFC-0208 (**QUEUE V closed**); Pack 0 **DONE** @ [`repair-package-0.md`](repair-package-0.md) / RFC-0207; trace [`product-requirements-trace.md`](product-requirements-trace.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase W `#331`–`#342`.  
**Closure:** consolidating **RFC-0215** reserved file-free until `#342` (id confirmed free 2026-09-11).  
**Не змінює:** анти-місію README; історію Phase V як never-DONE; C0/C1 `Calculate 2 + 2` як execution-basic для unconstrained math; `aira-core` ledger / LLM-in-Core; GUI stack (egui) rewrite; Pack 2 multi-model catalog / Work «Порівняти» (M1–M6); Pack 3–7; public bind / auto-trust як Desktop default.

> Примітка: аудит і `aira-repair.md` поза git-каноном (`p-only/`). Канон у репо = цей план + Pack 1 close + QUEUE + RFC-D.

---

## 0. Навіщо (після QUEUE V closed)

Phase V закрила **структурний** Pack 1 (snapshot persist, reuse key, activate hash continuity, executor field stamps, §6 e2e subset) і consolidating RFC-0208. Аудит @ `d1115f2` фіксує: **QUEUE V = DONE**, але продуктове приймання Pack 1 = **PARTIAL** — знімок зберігається, проте низка обов’язкових вимог не виконується і не відхиляється; є шлях до VERIFIED для підміненої арифметики (D1).

Головні залишкові дефекти (аудит D1–D6):

1. **D1** — `is_math_eval_safe` приймає `9-3` / `9/3` / `42`, reduction default → capsule `2+2` → VERIFIED 4.
2. **D2** — більшість `AdmissionConstraints` лише в reuse key; не enforce / не reject.
3. **D3** — HTTP/Serde мовчки ігнорує unknown nested fields.
4. **D4** — activation pointer facts штампуються як used-model без verified process binding.
5. **D5** — unsigned `VerifiedPointer` + спільна підміна weights+hash обходить нові checks.
6. **D6** — result read / reuse candidate / snapshot↔text / verify capsule не перевіряють повний task binding.

```text
QUEUE V closed (#323–#330 / RFC-0208)
  + audit d1115f2: Pack 1 PARTIAL (D1–D6)
  → Phase W IN PROGRESS: residual honesty A0–A10
    → не Pack 2 GUI (M1–M6); не Pack 3–7; не Core rewrite
```

---

## 1. Мета

1. Уточнити статус Pack 1: V structural DONE; residual honesty = Phase W (без переписування історії).
2. Math Problem → Capsule зберігає зміст; unsupported не стає `2+2`.
3. Unknown/invalid submit fields → явний 4xx до execution.
4. Обов’язкові constraints: supported→enforced або explicit unsupported reject.
5. Admission snapshot узгоджений із text/hash на кожному input boundary.
6. Activation authority = primary signed verify evidence (не unsigned pointer).
7. Production activation без implicit local-test trust.
8. Безпечна materialization weights (no-follow, post-copy, bounded buffer).
9. Backend виконує verified binding; mock ≠ used-model.
10. Reuse candidate і result/verify читання перевіряють task binding незалежно від index.

Цільовий ярлик: **Pack 1 residual honesty (A0–A10)** — не product-complete multi-model GUI (M1–M6 / Pack 2).

---

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` **не** отримує GUI / Help / egui / ledger / LLM runtime.
3. Phase V / RFC-0208 **не** переписуються як never-DONE.
4. Unconstrained C1 `Calculate 2 + 2` лишається deterministic execution-basic.
5. Choice of executor **never** grants `VERIFIED`.
6. Help офлайн без LLM; F1 не наказує недоступних дій (точкове узгодження в A0 / пізніше Pack 2).
7. Не редагувати `Manifesto etc/`, `Meditation_About/`.
8. Admission/activation evidence **не** послаблюється заради UX.
9. Не робити public bind / auto-trust default Desktop.
10. Не маскувати mock-execution як verified LLM output.
11. Schema/CLI DONE ≠ product DONE (Pack 0 / SCD §17).

---

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Pack 2 M1–M6 (inventory GUI / Compare / installed two-model acceptance)
Pack 3 ratings / Pack 4 remote / Pack 5 weight transfer / Pack 6 federation / Pack 7 product profile
Rewriting Phase V history as never-DONE
Public bind / auto-trust як default
Weakening observe / activate admission hash
Inventing full S1–S18 as one PR (лише окремі S* коли блокують A7/A8)
```

---

## 4. Смуги

```text
W0 Wiring (+ A0 contract honesty tips)
  → W1 Math capsule fidelity (D1)
    → W1 Strict submit decoding (D3)
      → W1 Constraints enforce-or-reject (D2)
        → W1 Admission boundary verify (D6 partial)
          → W1 Activate evidence authority (D5)
            → W1 Production trust (no implicit test key)
              → W1 Safe weights materialization
                → W1 Backend verified binding (D4)
                  → W1 Reuse candidate independent check
                    → W1 Result/verify task binding
                      → W close RFC-0215
```

---

## 5. Атоми → QUEUE `#331`–`#342`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#331` | W0 | Phase W wiring + A0 tips | план IN PROGRESS; QUEUE `#331`–`#342`; `phase_w_doc`; RFC-0215 reserved; Pack 1 PARTIAL honesty без rewrite V | math (#332) |
| `#332` | W1 | Math capsule fidelity | `9-3`→6, `9/3`→3, `42`→42; unsupported ≠ default `2+2`; RFC-0216 | strict HTTP (#333) |
| `#333` | W1 | Strict submit decoding | unknown nested fields / invalid → 4xx; text-only OK; RFC-0217 | constraints matrix (#334) |
| `#334` | W1 | Constraints enforce-or-reject | кожне поле: enforced або explicit unsupported; RFC-0218 | admission boundary (#335) |
| `#335` | W1 | Admission boundary verify | text/hash/kind/context mismatch reject; RFC-0219 | activate evidence (#336) |
| `#336` | W1 | Activate evidence authority | pointer locator-only; primary evidence authority; RFC-0220 | production trust (#337) |
| `#337` | W1 | Production activation trust | no implicit local-test; fixtures separate; RFC-0221 | materialize (#338) |
| `#338` | W1 | Safe weights materialization | no-follow + post-copy + bounded buffer; RFC-0222 | backend binding (#339) |
| `#339` | W1 | Backend verified binding | mismatch reject; mock ≠ used-model; RFC-0223 | reuse candidate (#340) |
| `#340` | W1 | Reuse candidate independent check | чужий VRA у ключі ≠ Completed; RFC-0224 | result binding (#341) |
| `#341` | W1 | Result/verify task binding | swap refs reject; generate ≠ false VerificationFailed; RFC-0225 | close (#342) |
| `#342` | W | RFC-0215 + close | consolidating; QUEUE W closed; Pack 1 residual honesty DONE | Pack 2 M* |

```text
#331 W0 wiring + A0 (DONE this PR → first OPEN #332)
  → #332 W1 math capsule
    → #333 W1 strict submit
      → #334 W1 constraints enforce-or-reject
        → #335 W1 admission boundary
          → #336 W1 activate evidence
            → #337 W1 production trust
              → #338 W1 safe materialize
                → #339 W1 backend binding
                  → #340 W1 reuse candidate
                    → #341 W1 result/verify binding
                      → #342 W close RFC-0215
```

**Anti-merge:** не зливати math із strict HTTP; не зливати evidence authority з backend binding; не зливати close з runtime; не зливати M1–M6 у W.

---

## 6. Acceptance Phase W (після close)

```text
QUEUE `#331`–`#342` DONE; no OPEN W atoms.
D1–D6 residual Pack 1 honesty closed per A1–A10.
Unconstrained C1 2+2 still VERIFIED via execution-basic.
Mock remains explicit; used-model requires verified binding.
Pack 2 M1–M6 still separate.
Phase V / RFC-0208 history preserved as structural DONE.
aira-core still has no GUI/ledger/LLM deps.
Анти-місія не порушена.
```

---

## 7. RFC (після активації)

- Consolidating (`#342`): **AIRA-RFC-0215** — reserved file-free until close (id confirmed free 2026-09-11).
- Per-atom RFC-D: `#332`→**0216** … `#341`→**0225** (призначаються на кожному PR); `#342`→0215 consolidating.

---

## 8. Activation rule

```text
QUEUE V closed @ RFC-0208
  + audit d1115f2 Pack 1 PARTIAL (D1–D6)
  + developer «активувати residual phase» (2026-09-11)
  → wiring `#331` DONE; план IN PROGRESS; first OPEN `#332`.
```

---

## 9. Матриця audit → атоми

| Аудит | Атом |
|-------|------|
| A0 contract / Pack 1 PARTIAL honesty | `#331` |
| A1 / D1 math capsule | `#332` |
| A2 / D3 strict submit | `#333` |
| A3 / D2 constraints | `#334` |
| A4 admission boundary | `#335` |
| A5 / D5 activate evidence | `#336` |
| A6 production trust | `#337` |
| A7 safe materialize | `#338` |
| A8 / D4 backend binding | `#339` |
| A9 reuse candidate | `#340` |
| A10 / D6 result+verify binding | `#341` |
| Close | `#342` |
| M1–M6 / D7 GUI | **out of W** (Pack 2) |
| S1–S18 | окремо коли блокують A7/A8; не весь список у W |
