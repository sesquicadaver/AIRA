# Phase X — Pack 2: local multi-model GUI (post-W)

**Статус:** **IN PROGRESS** (активовано `#343` wiring; `#343`–`#351` **DONE**; first OPEN `#352`; RFC-0226 reserved file-free until `#358`).  
**Джерела:** GUI-аудит `p-only/aira-current.md` (2026-09-13, скріншоти Desktop); Repair Pack 2 `p-only/aira-repair.md` §«Пакет 2»; audit M1–M6 `p-only/AIRA-audit-d1115f2-2026-09-11.md`; trace [`product-requirements-trace.md`](product-requirements-trace.md) (PR-M1…PR-M4); Phase W residual honesty [`phase-w-plan.md`](phase-w-plan.md) `#331`–`#342` **DONE** @ RFC-0215; **QUEUE W closed**.  
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
  → Phase X Pack 2: local multi-model GUI (IN PROGRESS @ `#343`)
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
2. First OPEN репо = `#352` після `#351` Network/address honesty; лінійність `#N` після `#N-1`.
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
| `#343` | X0 | Phase X wiring | **DONE**: план IN PROGRESS; QUEUE tip; `phase_x_doc`; RFC-0226 reserved | inventory (#344) |
| `#344` | X1 | Per-model inventory lifecycle (M1) | **DONE** @ RFC-0227: A/B slots; latest ≠ sole auth; restart; `phase_x_doc` | select API (#345) |
| `#345` | X1 | Model select API Auto/required (M2) | **DONE** @ RFC-0228: `select_model` Auto/Required; explained unready/removed | profile (#346) |
| `#346` | X1 | Request profile → snapshot (M3) | **DONE** @ RFC-0229: freeze allowed/excluded; mid-flight ≠ admitted | CLI (#347) |
| `#347` | X1 | CLI supported-contract parity (M4) | **DONE** @ RFC-0230: excluded flag; no unsupported no-op flags | Settings GUI (#348) |
| `#348` | X2 | Settings Models catalog GUI | **DONE** @ RFC-0231: list/scan/add/select/prepare + ready reason | Work executor (#349) |
| `#349` | X2 | Work executor + capability readiness | **DONE** @ RFC-0232: Auto/specific; math≠generate; pre-submit | mock/result (#350) |
| `#350` | X2 | Mock honesty + result triple | **DONE** @ RFC-0233: demo banner; requested/applied/executed | network UX (#351) |
| `#351` | X2 | Network/address honesty + P UX | **DONE** @ RFC-0234: loopback; HTTP≠peer≠advertised; P names; P3\|P4 radio | copy/IA (#352) |
| `#352` | X2 | Human copy + Settings≠System IA | локалізація; Зупинити вузол/Оновити стан/Завершити AIRA; edit у Параметрах; RFC-0235 | Help (#353) |
| `#353` | X2 | Help F1 model path | UK/EN офлайн; шлях моделі; без недоступних наказів; RFC-0236 | Compare (#354) |
| `#354` | X2 | Work Compare mode | дві моделі / без silent substitute; не Pack 3 ratings; RFC-0237 | data paths (#355) |
| `#355` | X2 | Model data paths (light) | каталог зберігання + зайнятий/доступний простір у Settings; RFC-0238 | apply-diff (#356) |
| `#356` | X2 | Apply-diff honesty | змінено / застосовано / треба restart + помилка apply; RFC-0239 | M6 (#357) |
| `#357` | X3 | Two-model installed acceptance (M6) | GUI+CLI+HTTP; cold restart; unavailable fail-closed; RFC-0240 | close (#358) |
| `#358` | X | RFC-0226 + close | consolidating; QUEUE X closed; Pack 2 DONE | Pack 3+ |

```text
#343 X0 wiring DONE
  → #344 X1 inventory M1 DONE @ RFC-0227
    → #345 X1 select API M2 DONE @ RFC-0228
      → #346 X1 profile→snapshot M3 DONE @ RFC-0229
        → #347 X1 CLI parity M4 DONE @ RFC-0230
          → #348 X2 Settings Models GUI DONE @ RFC-0231
            → #349 X2 Work executor + readiness DONE @ RFC-0232
              → #350 X2 mock + result DONE @ RFC-0233
                → #351 X2 network/address + P UX DONE @ RFC-0234
                  → #352 X2 copy + IA (first OPEN)
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
  + developer / wiring `#343` DONE
  + джерела: aira-current.md + Pack 2 + M1–M6
  → phase-x-plan **IN PROGRESS**; `#343`–`#351` DONE; first OPEN `#352`.
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
| High: Робота виконавець + readiness math≠generate | `#349` **DONE** |
| High: Mock honesty + result triple | `#350` **DONE** |
| High: Network/address honesty + P UX | `#351` **DONE** |
| High: peer_listen 127.0.0.1 honesty; address roles; P UX | `#351` **DONE** |
| Med: локалізація; Стоп/Вийти; Settings≠System | `#352` |
| F1 model path | `#353` |
| Pack 2 Compare / PR-M4 | `#354` |
| Med: каталог/простір моделей | `#355` |
| Med: apply-diff honesty | `#356` |
| M6 two-model acceptance | `#357` |
| Close | `#358` |
| Pack 3 ratings / Pack 4 quotas / Pack 5 weights share | **out of X** |
| W residual D1–D6 | **Phase W** `#333`–`#342` |
