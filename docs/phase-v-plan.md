# Phase V — Repair Pack 1: admission integrity & binding

**Статус:** **IN PROGRESS** (активовано 2026-09-10; `#323`–`#328` **DONE**; first OPEN `#329`).  
**Джерела:** Repair Pack 1 `p-only/aira-repair.md` §«Пакет 1»; Pack 0 **DONE** @ [`repair-package-0.md`](repair-package-0.md) / RFC-0207; Phase U **DONE** @ [`phase-u-plan.md`](phase-u-plan.md) / RFC-0198 (**QUEUE U closed**); trace [`product-requirements-trace.md`](product-requirements-trace.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase V `#323`–`#330`.  
**Closure:** consolidating **RFC-0208** reserved file-free until `#330` (id confirmed free 2026-09-10).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core` ledger / LLM-in-Core; GUI stack (egui); IA Work / System / Settings + Help·F1; Pack 2 multi-model catalog GUI; remote compute Pack 4; weight transfer Pack 5; public bind / auto-trust як Desktop default; admission hash policy (`check_activated`) не послаблюється.

> Примітка: `aira-repair.md` / `aira-current.md` поза git-каноном (`p-only/`). Канон у репо = цей план + Pack 0 trace + QUEUE + RFC-D.

---

## 0. Навіщо (після Pack 0 / U)

Pack 0 зафіксував продуктове право на вибір моделі/ресурсів. Phase U замкнула cross-path honesty результатів. Але **вибір користувача ще не проходить незмінним знімком** від UI/CLI/HTTP до reuse і виконавця:

1. Submit приймає лише `text` — немає immutable admission snapshot (модель/набір, content identity, gen params, placement, privacy/budget, fallback, require-new vs allow-reuse).
2. Reuse-index ключується лише hash тексту → результат іншої моделі може «задовольнити» вимогу виконати моделлю X.
3. `activate_verified()` хешує після копіювання **без** звіряння з `VerifiedPointer.content_hash`.
4. Problem→Capsule→Output→Result / executor facts для generate-local не замкнуті як обов’язковий binding path.

```text
QUEUE U closed (#313–#322 / RFC-0198)
  + Pack 0 DONE (RFC-0207)
  → Phase V IN PROGRESS: admission integrity (Repair Pack 1)
    → не Core rewrite; не GPU marketplace; не Pack 2 GUI catalog
```

---

## 1. Мета

1. До admission — **immutable snapshot** користувацьких вимог (розширення контракту запиту, не нова Core-сутність).
2. Зміна Settings під час виконання **не** змінює вже прийняте завдання.
3. Reuse застосовується **після** snapshot-умов; порівняння/вимір швидкості → require-new-execution.
4. `activate_verified` fail-closed при mismatch verify→activate content hash.
5. Binding Problem→Capsule→Output→Result + підтверджені execution facts (обрано ≠ виконано без доказу).
6. Обмеження не обходяться іншим frontend, невідомим полем, fallback, reuse або mutable pointer.

Цільовий ярлик: **Reference v0.3-strict + admission integrity (Pack 1)** — не product-complete multi-model GUI (Pack 2).

---

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` **не** отримує GUI / Help / egui / ledger / LLM runtime.
3. IA Phase O **не** переписується.
4. C1 `Calculate 2 + 2` → `execution-basic`.
5. Choice of executor **never** grants `VERIFIED`.
6. Help офлайн без LLM.
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
Pack 2 Settings catalog / Work «Порівняти» GUI
Pack 3 ratings measurement UI
Pack 4 remote compute cycle
Pack 5 weight transfer ≠ activated pointer
Public bind / auto-trust як default
Weakening observe / activate admission hash
```

---

## 4. Смуги

```text
V0 Wiring (план у QUEUE + living smoke; RFC-0208 reserved)
  → V1 Admission snapshot type + persist-on-admit
    → V1 Submit API carries constraints (CLI/HTTP/Desktop)
      → V1 Reuse after constraints
        → V1 activate_verified hash continuity
          → V1 Capsule↔Output↔Result binding (LLM path)
            → V2 End-to-end fail-closed tests (repair §6 subset)
              → V close RFC-0208 + repair-package-1
```

---

## 5. Атоми → QUEUE `#323`–`#330`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#323` | V0 | Phase V wiring | план IN PROGRESS; QUEUE `#323`–`#330`; `phase_v_doc`; tips; RFC-0208 reserved file-free | snapshot type (#324) |
| `#324` | V1 | Admission snapshot type + persist | **DONE** @ RFC-0209: immutable fields + persist on admit | Submit API surface (#325) |
| `#325` | V1 | Submit API carries constraints | **DONE** @ RFC-0210: HTTP/CLI/Desktop beyond `text`; mid-run Settings ≠ mutate | reuse key (#326) |
| `#326` | V1 | Reuse after constraints | **DONE** @ RFC-0211: reuse key ⊇ snapshot; model-X miss ≠ text-only hit; require-new | activate hash (#327) |
| `#327` | V1 | `activate_verified` hash continuity | **DONE** @ RFC-0212: post-copy == VerifiedPointer.content_hash; tamper → reject | capsule binding (#328) |
| `#328` | V1 | Capsule↔Output↔Result binding | **DONE** @ RFC-0213: executor facts stamped; CapsuleCompleted [output,capsule] | e2e suite (#329) |
| `#329` | V2 | E2E fail-closed tests | reuse/model, settings-during-run, verify-tamper-before-activate | Pack 2 GUI |
| `#330` | V | RFC-0208 + close | consolidating; QUEUE V closed; `repair-package-1.md` DONE | Pack 2 |

```text
#323 V0 wiring (DONE)
  → #324 V1 admission snapshot DONE @ RFC-0209
    → #325 V1 submit constraints API DONE @ RFC-0210
      → #326 V1 reuse after constraints DONE @ RFC-0211
        → #327 V1 activate hash continuity DONE @ RFC-0212
          → #328 V1 capsule/result binding DONE @ RFC-0213
            → #329 V2 e2e fail-closed (first OPEN)
              → #330 V close RFC-0208
```

**Anti-merge:** не зливати snapshot type з Submit API; не зливати reuse з activate hash; не зливати binding з Pack 2 GUI; не зливати close з runtime atoms.

---

## 6. Acceptance Phase V (після close)

```text
QUEUE `#323`–`#330` DONE; no OPEN V atoms.
Admitted task keeps snapshot if Settings change mid-run.
Same text + different model/constraints → no incorrect reuse hit.
Verify-file tampered before activate → activation rejected.
Generate-local path stamps real executor/model facts; mock remains explicit.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no GUI/ledger/LLM deps.
Анти-місія не порушена; Pack 2 GUI still separate.
```

---

## 7. RFC (після активації)

- Consolidating (`#330`): **AIRA-RFC-0208** — reserved file-free until close (id confirmed free 2026-09-10).
- Per-atom RFC-D: `#324`→**0209** … `#329`→**0214** (призначаються на кожному PR); `#330`→0208 consolidating.

---

## 8. Activation rule

```text
QUEUE U closed @ RFC-0198
  + Pack 0 DONE @ RFC-0207
  + developer «Pack 1» / «активувати Phase V» (2026-09-10)
  → wiring `#323` DONE; `#324`–`#328` DONE @ RFC-0209…0213; план IN PROGRESS; first OPEN `#329`.
```

---

## 9. Матриця evidence → атоми (`aira-repair.md` Pack 1)

| Вимога | Атом |
|--------|------|
| Immutable admission snapshot | `#324`–`#325` |
| Settings mid-run ≠ mutate task | `#325` (+ tests `#329`) |
| Reuse after constraints / require-new | `#326` |
| verify→activate content hash | `#327` |
| Problem→Capsule→Output→Result + executor | `#328` |
| Cross-path bypass fail-closed | `#329` |
| Pack 1 close docs | `#330` |
