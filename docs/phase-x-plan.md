# Phase X — Pack 2: local multi-model GUI (post-W)

**Статус:** **QUEUED** (записано 2026-09-13; **не** first OPEN; виконувати лише після **QUEUE W closed** / `#342`; активація = `#343` wiring).  
**Джерела:** GUI-аудит `p-only/aira-current.md` (2026-09-13, скріншоти Desktop); Repair Pack 2 `p-only/aira-repair.md` §«Пакет 2»; audit M1–M6 `p-only/AIRA-audit-d1115f2-2026-09-11.md`; trace [`product-requirements-trace.md`](product-requirements-trace.md) (PR-M1…PR-M4); Phase W residual honesty [`phase-w-plan.md`](phase-w-plan.md) `#331`–`#342` (**IN PROGRESS**; first OPEN `#336`; RFC-0215 reserved).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase X `#343`–`#358`.  
**Closure:** consolidating **RFC-0226** reserved file-free until `#358` (id confirmed free 2026-09-13).  
**Не змінює:** анти-місію README; Phase W / V історію; Pack 1 admission honesty; `aira-core` ledger / LLM-in-Core; egui stack rewrite; Pack 3 ratings evidence UI; Pack 4 remote compute quotas; Pack 5 weight-transfer ≠ inference; Pack 6–7; public bind / auto-trust як Desktop default.

> Примітка: `p-only/aira-current.md` / `aira-repair.md` / аудити поза git-каноном. Канон у репо = цей план + QUEUE + RFC-D + tip у `phase-w-plan.md` §handoff.

---

## 0. Навіщо (після Pack 1 honesty)

Phase W закриває **residual honesty** Pack 1 (D1–D6 / A1–A10). Скріншоти Desktop + `aira-current.md` фіксують іншу прогалину: **GUI показує статуси, але не дає завершити сценарій з моделями**.

Найважливіша діра (канон Pack 2):

```text
додати/виявити модель → підготувати → вибрати → виконати задачу → побачити фактичного виконавця
```

без обов’язкового CLI. Audit M1–M6 + repair Pack 2 — той самий критерій: дві реальні моделі одним штатним шляхом GUI/CLI/HTTP; cold restart зберігає вибір; unavailable ≠ silent fallback.

```text
QUEUE W (#331–#342 / RFC-0215) → close
  + aira-current.md + Pack 2 / M1–M6
  → Phase X QUEUED: local multi-model GUI
    → не Pack 3 ratings; не Pack 4–5 share; не Core rewrite
```

---

## 1. Мета

1. Per-model inventory lifecycle: verified/available незалежно; `activated.latest` = default pointer, **не** global execution authority.
2. Select API: Auto / required model; unready/removed → пояснена помилка.
3. Request/profile overrides → immutable admission snapshot (без mid-flight mutate).
4. CLI parity для реально підтримуваних полів (без no-op flags).
5. Settings→Моделі: список, виявлення/додавання, вибір, підготовка, конкретна причина неготовності — **без CLI**.
6. Робота: вибір виконавця (Авто / конкретна / Порівняти); readiness **по можливості** (math ≠ generate); перед запуском — фактичні умови.
7. Mock executor — помітний демо-статус + дія налаштування; результат показує requested ≠ applied ≠ executed.
8. Мережа: loopback listen чесно названий; HTTP / peer_listen / advertised розведені; P0–P6 людські назви; P3|P4 mutex UI.
9. Локалізація користувацьких рядків; кнопки програми = фактична поведінка; редагування в Параметрах, спостереження в Стані.
10. F1 офлайн відповідає доступним діям шляху моделі.
11. Installed-product acceptance двох моделей (M6).

Цільовий ярлик: **Pack 2 local multi-model GUI (first user-complete result)** — не Pack 3–7.

---

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. First OPEN лишається Phase W `#333`…`#342`, доки W не closed; `#343+` у хвості.
3. `aira-core` **не** отримує GUI / Help / egui / ledger / LLM runtime.
4. Choice of executor **never** grants `VERIFIED`.
5. Mock ≠ used-model; не маскувати mock як verified LLM.
6. Help офлайн без LLM; F1 не наказує недоступних дій.
7. Не редагувати `Manifesto etc/`, `Meditation_About/`.
8. Admission/activation evidence **не** послаблюється заради UX.
9. Не робити public bind / auto-trust default Desktop.
10. Schema/CLI DONE ≠ product DONE (Pack 0 / SCD §17).
11. Не дублювати Model Registry: `model_ref` + inventory artifacts.

---

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Pack 3 ratings / evidence measurement UI
Pack 4 remote compute + OS quota enforcement product
Pack 5 weight transfer path ≠ activated publish
Pack 6 federation product / Pack 7 profile
Rewriting Phase W/V as never-DONE
Public bind / auto-trust як default
Weakening observe / activate admission hash
Full resource-share permissions UI (aira-current «Надання ресурсів»)
Inventing S1–S18 wholesale
```

---

## 4. Смуги

```text
X0 Wiring (після QUEUE W closed)
  → X1 Inventory lifecycle (M1)
    → X1 Select API Auto/required (M2)
      → X1 Profile → snapshot (M3)
        → X1 CLI contract parity (M4)
          → X2 Settings Models catalog GUI
            → X2 Work executor + capability readiness
              → X2 Mock honesty + result triple
                → X2 Network/address honesty + P-profile UX
                  → X2 Human copy + Settings≠System IA
                    → X2 Help F1 model path
                      → X2 Compare mode (no silent substitute)
                        → X3 Two-model acceptance (M6)
                          → X close RFC-0226
```

---

## 5. Атоми → QUEUE `#343`–`#358`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#343` | X0 | Phase X wiring | план **IN PROGRESS**; QUEUE tip; `phase_x_doc`; RFC-0226 reserved; handoff з W | inventory (#344) |
| `#344` | X1 | Per-model inventory lifecycle (M1) | A/B verified/available незалежно; latest ≠ global exec authority; restart зберігає обидві; RFC-0227 | select API (#345) |
| `#345` | X1 | Model select API Auto/required (M2) | Auto + required A/B; unready/removed → пояснена помилка; RFC-0228 | profile (#346) |
| `#346` | X1 | Request profile → snapshot (M3) | overrides/excludes; mid-flight Settings ≠ admitted; RFC-0229 | CLI (#347) |
| `#347` | X1 | CLI supported-contract parity (M4) | підтримувані поля доступні; no-op flags відсутні; RFC-0230 | Settings GUI (#348) |
| `#348` | X2 | Settings Models catalog GUI | список/scan/add/select/prepare + ready reason без CLI; RFC-0231 | Work executor (#349) |
| `#349` | X2 | Work executor + capability readiness | Авто/конкретна; math vs generate readiness; pre-submit умови; RFC-0232 | mock/result (#350) |
| `#350` | X2 | Mock honesty + result triple | помітний демо-виконавець; requested/applied/executed; RFC-0233 | network UX (#351) |
| `#351` | X2 | Network/address honesty + P UX | loopback пояснено; HTTP≠peer≠advertised; P0–P6 назви; P3\|P4 radio; RFC-0234 | copy/IA (#352) |
| `#352` | X2 | Human copy + Settings≠System IA | локалізація; Зупинити вузол/Оновити стан/Завершити AIRA; edit у Параметрах; RFC-0235 | Help (#353) |
| `#353` | X2 | Help F1 model path | UK/EN офлайн; шлях моделі; без недоступних наказів; RFC-0236 | Compare (#354) |
| `#354` | X2 | Work Compare mode | дві моделі / без silent substitute; не Pack 3 ratings; RFC-0237 | data paths (#355) |
| `#355` | X2 | Model data paths (light) | каталог зберігання + зайнятий/доступний простір у Settings; RFC-0238 | apply-diff (#356) |
| `#356` | X2 | Apply-diff honesty | змінено / застосовано / треба restart + помилка apply; RFC-0239 | M6 (#357) |
| `#357` | X3 | Two-model installed acceptance (M6) | GUI+CLI+HTTP; cold restart; unavailable fail-closed; RFC-0240 | close (#358) |
| `#358` | X | RFC-0226 + close | consolidating; QUEUE X closed; Pack 2 DONE | Pack 3+ |

```text
#343 X0 wiring (після W close)
  → #344 X1 inventory M1
    → #345 X1 select API M2
      → #346 X1 profile→snapshot M3
        → #347 X1 CLI parity M4
          → #348 X2 Settings Models GUI
            → #349 X2 Work executor + readiness
              → #350 X2 mock + result triple
                → #351 X2 network/address + P UX
                  → #352 X2 copy + IA
                    → #353 X2 Help F1 model
                      → #354 X2 Compare
                        → #355 X2 model data paths
                          → #356 X2 apply-diff honesty
                            → #357 X3 M6 acceptance
                              → #358 X close RFC-0226
```

**Anti-merge:** не зливати inventory із Settings GUI; не зливати Work executor із Compare; не зливати M6 із close; не зливати Pack 3 ratings / Pack 4–5 у X; **не** вставляти X-атоми між `#333`–`#342`.

---

## 6. Acceptance Phase X (після close)

```text
QUEUE `#343`–`#358` DONE; no OPEN X atoms.
Наскрізний шлях GUI без CLI: discover→prepare→select→run→see executor.
M1–M6 / Pack 2 repair criterion closed (`#357` = M6).
PR-M1 product path; PR-M3 GUI triple; PR-M4 Compare without silent substitute.
Mock explicit; VERIFIED не від вибору моделі.
Pack 3–7 still separate.
Phase W / RFC-0215 history preserved.
aira-core still has no GUI/ledger/LLM deps.
Анти-місія не порушена.
```

---

## 7. RFC

- Consolidating (`#358`): **AIRA-RFC-0226** — reserved file-free until close (id confirmed free 2026-09-13).
- Per-atom RFC-D: `#344`→**0227** … `#356`→**0239**, `#357`→**0240** (на кожному PR); `#358`→0226 consolidating.

---

## 8. Activation rule

```text
QUEUE W closed @ RFC-0215
  + developer / wiring `#343`
  + джерела: aira-current.md + Pack 2 + M1–M6
  → phase-x-plan IN PROGRESS; first OPEN `#344` (після `#343` DONE).
До `#342` DONE: Phase X = QUEUED; first OPEN лишається `#336`.
```

---

## 9. Матриця aira-current / audit → атоми

| Джерело | Атом |
|---------|------|
| M1 inventory lifecycle | `#344` |
| M2 select API | `#345` |
| M3 profile→snapshot | `#346` |
| M4 CLI parity | `#347` |
| High: Параметри→Моделі GUI | `#348` |
| High: Робота виконавець + readiness math≠generate | `#349` |
| High: mock помітний; result triple | `#350` |
| High: peer_listen 127.0.0.1 honesty; address roles; P UX | `#351` |
| Med: локалізація; Стоп/Вийти; Settings≠System | `#352` |
| F1 model path | `#353` |
| Pack 2 Compare / PR-M4 | `#354` |
| Med: каталог/простір моделей | `#355` |
| Med: apply-diff honesty | `#356` |
| M6 two-model acceptance | `#357` |
| Close | `#358` |
| Pack 3 ratings / Pack 4 quotas / Pack 5 weights share | **out of X** |
| W residual D1–D6 | **Phase W** `#333`–`#342` |
