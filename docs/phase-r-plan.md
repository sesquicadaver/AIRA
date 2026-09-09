# Phase R — Actionable Desktop Connection UX (post-Q)

**Статус:** складено 2026-09-09; **IN PROGRESS** — QUEUE `#286`–`#292` **DONE**; first OPEN `#293`; RFC-0174 reserved file-free.  
**Джерела:** post-Q UX review 2026-09-08…09 (Developer Preview cold-start / Connection / Help); канон [`desktop-ux.md`](desktop-ux.md); Phase O [`phase-o-plan.md`](phase-o-plan.md) **DONE** @ RFC-0146; Phase P [`phase-p-plan.md`](phase-p-plan.md) **DONE** @ RFC-0156; Phase Q [`phase-q-plan.md`](phase-q-plan.md) **DONE** @ RFC-0164; QUEUE Q closed.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase R `#286`–`#294`.  
**Closure:** consolidating **RFC-0174** (id **confirmed free** 2026-09-09; file-free until `#294`).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core`; IA Work / System / Settings + Help·F1; семантику P0–P6; UNKNOWN≠OFFLINE; AddressBook≠сесії; Applied≠Saved; Phase P honesty-факти.

> Примітка: канон у репо = цей план + `desktop-ux.md` + QUEUE + RFC-D.

---

## 0. Опис проблеми

### 0.1 Симптом

Користувач бачить **багато тексту** (status strip, System → З’єднання, mesh-банер, Saved/Applied, footer `help:`/`try:`, F1), але **не отримує наступного кроку**, щоб стати reachable до іншого учасника. У новому/порожньому профілі це виглядає як «непотріб з технічними поясненнями», хоча розділ Settings **є**.

### 0.2 Коренева причина

Phase O/P успішно винесли **honesty** у primary UI (не брехати: UNKNOWN≠OFFLINE, книга≠сесії, Applied лише з runtime). Job-to-be-done «під’єднатись» лишився:

1. під **«Технічні подробиці»** (`ui_network_ops`: профіль P0–P2, invite/QR);
2. у Help як **захист від брехні** («розумій контракт»), а не сценарій;
3. з CTA **Оновити** там, де потрібен шлях invite → профіль з peer → Stop/Start.

Тобто: **немає браку тексту** — є **неправильний пріоритет тексту** (explain ≫ instruct).

### 0.3 Evidence (код / docs)

| Спостереження | Де |
|---------------|-----|
| Connect-контроли під collapsing tech | `crates/aira-desktop/src/app/ui.rs` — Connection → `sys_tech_details` → `ui_network_ops` |
| Primary CTA для Unknown/LocalOnly = Refresh | той самий `ui.rs` |
| Сирий banner `UNKNOWN` / `LOCAL ONLY` поруч із людським conclusion | mesh render + `system_view` |
| Help connect: «Технічні подробиці» + «розумій контракт» | `docs/help/uk/network.connect.md` |
| Problem footer показує wire-id `help:` / `try:` | `ui.rs` problem strip |
| UK: частина mesh/discovery labels лишається EN | `i18n.rs` |

### 0.4 Що *не* є цією проблемою

- Зникнення меню Settings (IA на місці).
- Сам факт `data-root` (різний root = різний профіль — by design).
- Honesty-семантика Phase P (її **зберігаємо**, змінюємо **подачу**).
- Portable USB / absolute `http_token_ref` — **поза** Phase R (окремий план за потреби).

```text
QUEUE Q closed (#275–#285 / RFC-0164)
  → Phase R IN PROGRESS: actionable Connection UX без ослаблення honesty
    → не нові вкладки; не Core rewrite; не GPU marketplace
```

---

## 1. Мета

1. На **System → З’єднання** користувач завжди бачить **один наступний крок** (CTA), узгоджений зі станом.
2. Канонічний сценарій видимий без обов’язкового розгортання tech:  
   **запрошення → профіль з peer (P1/P2) → Stop/Start → очікуваний стан**.
3. Honesty лишається: сирі enum / wire-id / операторські поля — **вторинні** (tech / monospace), не primary headline.
4. Status strip і Help ведуть до **дії**, не лише до діагнозу.
5. Cold-start / порожній профіль: зрозуміло «що доступно зараз» + «що зробити, щоб з’явилась мережа учасників» — без фейкового Applied/CONNECTED.

Цільовий ярлик: **Desktop Developer Preview + actionable Connection** (не WCAG; не web rewrite).

---

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR (після активації).
2. `aira-core` **не** отримує GUI / Help / egui.
3. IA Phase O (**Work / System / Settings + Help·F1**) **не** переписується на 4 вкладки.
4. C1 `Calculate 2 + 2` → `execution-basic`.
5. `DISCOVERED ≠ TRUSTED`; AddressBook ≠ живі сесії; UNKNOWN ≠ OFFLINE.
6. Saved ≠ Applied; Applied лише з confirmed runtime.
7. Help офлайн без LLM.
8. Не редагувати `Manifesto etc/`, `Meditation_About/`.
9. Не послаблювати reachability/trust evidence заради «зеленішої» кнопки.

---

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Нова вкладка Network (повернення E1 4-tab IA)
Фейковий CONNECTED / Applied без runtime
Public bind / auto-trust як default Desktop
Portable USB layout / absolute-path migration (окремий план)
Rewriting Phase O–Q as never-DONE
Cosmetic theme-only PRs
```

---

## 4. Смуги

```text
R0 Wiring (план у QUEUE + living smoke; RFC-0174 reserved)
  → R1 Connection next-step CTA matrix
    → R2 Promote invite + profile path (не лише tech collapse)
      → R3 Primary language: human conclusion > raw enum
        → R4 Problem footer / strip actionability
          → R5 Help connect/reachability/trust rewrite
            → R6 UK parity (mesh/discovery operator labels)
              → R7 Cold-start empty-profile guidance
                → R8 RFC-0174 close
```

---

## 5. Атоми → QUEUE `#286`–`#294` (після активації)

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#286` | R0 | Phase R wiring | **DONE**: план IN PROGRESS; QUEUE `#286`–`#294`; `phase_r_doc`; tips; RFC-0174 reserved file-free; fmt tip | CTA matrix (#287) |
| `#287` | R1 | Connection next-step CTA | **DONE**: стан → рівно один primary CTA (`connection_cta` + UI); RFC-0175 | promote controls (#288) |
| `#288` | R2 | Promote connect controls | **DONE**: `ui_connect_primary` (P0–P2+invite) поза tech; RFC-0176 | language (#289) |
| `#289` | R3 | Human-primary mesh/strip | **DONE**: `mesh_language` + human mesh banner; RFC-0177 | footer (#290) |
| `#290` | R4 | Actionable problem/strip | **DONE**: `problem_action` human next-step; RFC-0178 | Help (#291) |
| `#291` | R5 | Help connect scenario | **DONE**: `network.connect` (+ trust/reachability) = invite→profile→restart; RFC-0179 | UK parity (#292) |
| `#292` | R6 | UK mesh/discovery parity | **DONE**: EN==UK mesh/discovery chrome; tech-only CLI fields; RFC-0180 | cold-start (#293) |
| `#293` | R7 | Cold-start empty profile | новий root / P0: один guidance рядок + CTA без fake Applied/CONNECTED | RFC close (#294) |
| `#294` | R | RFC-0174 + close | consolidating RFC-0174; QUEUE R closed; no OPEN R atoms | — |

```text
#286 R0 wiring DONE
  → #287 R1 CTA matrix DONE
    → #288 R2 promote connect controls DONE
      → #289 R3 human-primary language DONE
        → #290 R4 problem/strip actions DONE
          → #291 R5 Help scenario DONE
            → #292 R6 UK parity DONE
              → #293 R7 cold-start OPEN
                → #294 R8 RFC-0174 close
```

---

## 6. Acceptance Phase R

```text
QUEUE `#286`–`#294` DONE; no OPEN R atoms (після активації й close).
На Connection cold-start UK: видимий наступний крок без обов’язкового Technical details.
Сценарій invite → P1/P2 → Stop/Start виконується з primary Connection surface.
UNKNOWN≠OFFLINE і book≠sessions збережені; немає fake CONNECTED.
Help network.connect описує сценарій, не лише «розумій контракт».
Strip не єдине джерело «потрібна дія» без CTA.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no GUI/ledger deps.
Анти-місія не порушена.
```

---

## 7. RFC

- Consolidating (`#294`): **AIRA-RFC-0174** — id **confirmed free** 2026-09-09 (file-free until close).
- Per-atom RFC-D (після активації): `#287`→**0175** … `#293`→**0181** (призначаються на кожному PR); `#294`→0174 consolidating.

---

## 8. Activation rule

```text
QUEUE Q closed @ RFC-0164
  + developer «активувати Phase R» 2026-09-09
  → wiring `#286` DONE; план IN PROGRESS; first OPEN `#287`.
```

---

## 9. Матриця CTA (`#287` **DONE** @ RFC-0175)

| Стан Connection | Primary CTA (приклад) | Не primary |
|-----------------|------------------------|------------|
| P0 / лише локально, хоче учасників | «Увімкнути приватну мережу» → P1/P2 | сирий UNKNOWN |
| Немає trust / порожня книга | «Імпортувати запрошення» | лише Refresh |
| Профіль змінено, RestartNeeded | «Стоп → Старт» | «Оновити» як єдине |
| Unknown / LocalOnly після P1+peer | Refresh **або** перевірка reachability + пояснення | fake OFFLINE |
| Direct/Relayed | стан OK; secondary: додати учасника | стіна disclaimers |

Honesty-речення лишаються **коротким secondary** («ще не перевірено — це не офлайн»).
