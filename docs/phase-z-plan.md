# Phase Z — Desktop chrome density (post-audit A)

**Статус:** **IN PROGRESS** (активовано `#390` wiring; consolidating **AIRA-RFC-0244** reserved, **confirmed free** 2026-09-25 until `#397` close).  
**Перший OPEN:** `#391` (first OPEN `#391`).  
**Джерела:** UX-скарги 2026-09-25 (Work chat / «Як це працює» / Connection spam); план [`.omx/plans/2026-09-25-desktop-gui-refinement.md`](../.omx/plans/2026-09-25-desktop-gui-refinement.md); канон [`desktop-ux.md`](desktop-ux.md); Phase R [`phase-r-plan.md`](phase-r-plan.md) **DONE** @ RFC-0174; Phase S [`phase-s-plan.md`](phase-s-plan.md) **DONE** @ RFC-0182; Audit A `#376`–`#389` **DONE**.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase Z `#390`–`#397`.  
**Closure:** consolidating **RFC-0244** @ `#397` (file-free until close).  
**Не змінює:** анти-місію README; C0/C1; `aira-core`; IA Work / System / Settings + Help·F1; UNKNOWN≠OFFLINE; setup≠remote; Applied лише після confirmed Start; cold_start forbids Connected; CTA-матрицю `connection_cta` як продукт; public bind / auto-trust default; Pack 3+.

> Канон у репо = цей план + `desktop-ux.md` + QUEUE + RFC-D. OMX draft — provenance.

---

## 0. Навіщо

Phase R/S винесли **honesty** у primary chrome (boundary always-on `#299`, cold-start guidance `#293`, Unknown≠Offline). Job-to-be-done «працювати» і «зробити наступний крок» програє **explain ≫ instruct**: мізерний Work editor, дубль «Як це працює» поруч із F1, три шари Connection copy на cold-start.

```text
Audit A closed (#376–#389; немає OPEN)
  → Phase Z IN PROGRESS: chrome density / progressive disclosure
    → не ослаблення honesty; не нова IA; не theme-only
```

---

## 1. Мета

1. Work: editor займає помітну частку viewport (`desired_rows` ≥ 8 або еквівалентний split).
2. Work: немає in-pane tutorial «Як це працює» — навчання через F1.
3. System → З’єднання: короткий human conclusion + одна CTA (+ опційно один short secondary ≤ ~80 символів); без always-on multi-paragraph `conn_boundary_guidance` + `conn_cold_start_guidance`.
4. Повна межа setup≠remote / loopback≠dial лишається в Help `network.connect`.
5. Density budget задокументований у `desktop-ux.md`.
6. Honesty-тести (`cold_start`, connection_cta) без семантичного ослаблення.

---

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` без GUI/Help/egui.
3. IA Phase O не переписується.
4. UNKNOWN ≠ OFFLINE; AddressBook ≠ живі сесії; Saved ≠ Applied.
5. Help офлайн без LLM.
6. Не редагувати `Manifesto etc/`, `Meditation_About/`.
7. Admission / cold_start forbids Connected **не** послаблюються.
8. Primary Connection CTA лишається з `connection_cta::primary_connection_cta`.

---

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite / нова вкладка Network
Public bind / auto-trust default
Ослаблення RFC-0186 семантики (лише relocation chrome → F1)
Pack 3+ / theme-only PRs
Зміна матриці CTA як окремий продукт у цьому етапі
```

---

## 4. Смуги

```text
Z0 Wiring (#390)
  → Z1 Work layout (#391)
    → Z2 Remove how-it-works (#392)
      → Z3 Short Unknown label (#393)
        → Z4 Boundary/cold off primary (#394)
          → Z5 Secondary copy pass (#395)
            → Z6 Honesty/Help regression (#396)
              → Z7 RFC-0244 close (#397)
```

---

## 5. Атоми → QUEUE `#390`–`#397`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#390` | Z0 | Phase Z wiring | **DONE**: plan IN PROGRESS; QUEUE tip `#391`; `phase_z_doc`; density contract у `desktop-ux`; RFC-0244 reserved file-free | layout/copy UI |
| `#391` | Z1 | Work layout budget | editor `desired_rows` ≥ 8 або пропорційний split; тест/константа | remove how-it-works |
| `#392` | Z2 | Remove «Як це працює» | немає `work_how_it_works` CollapsingHeader; F1 covers | Connection copy |
| `#393` | Z3 | Short `sys_conn_unknown` | короткий EN/UK label; тест довжини/змісту | boundary/cold remove |
| `#394` | Z4 | Boundary/cold off primary | не always-on повні guidance абзаци; Help лишає межу | CTA matrix change |
| `#395` | Z5 | Secondary copy pass | ≤1 secondary weak line / секція Work readiness & System extras | Settings deep forms |
| `#396` | Z6 | Honesty + Help regression | cold_start + connection_cta + help link check | new Help topic unless gap |
| `#397` | Z7 | RFC-0244 + close | consolidating RFC; tip → немає OPEN Z; phase complete | Pack 3+ |

---

## 6. Density budget (chrome)

**Instruct ≫ Explain** на primary chrome. Honesty не зникає — повні заперечення в F1.

| Surface | Primary | Max secondary guidance |
|---------|---------|------------------------|
| Work | editor + answer + controls | 0 in-pane tutorials; readiness reasons only when blocked |
| System → Connection | short conclusion + 1 CTA | ≤1 short line (≤ ~80 chars) or F1 link |
| Help F1 | full honesty / scenario | — |

Always-on multi-paragraph Connection guidance (historical `#293`/`#299` chrome) is **superseded as primary policy**; authority = Help + short status + CTA.

---

## 7. Verification

```text
cargo test -p aira-desktop-runtime --test phase_z_doc
cargo test -p aira-desktop --lib   # as scoped per atom
```

Manual smoke after Z1–Z4: Work editor visibly larger; no «Як це працює»; Connection cold-start = short status + CTA only.
