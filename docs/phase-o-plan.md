# Phase O — Desktop UX contract & contextual Help (F1)

**Статус:** складено 2026-09-06; **IN PROGRESS** — QUEUE `#255`–`#261` **DONE**; first OPEN `#262`.  
**Джерела:** UX draft 2026-09-06 (`aira-gui.md`, `aira-desktop-ux-help-draft.md`); канон [`desktop-ux.md`](desktop-ux.md); QUEUE N-fix closed @ RFC-0139.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase O `#255`–`#265`.  
**Closure:** consolidating **RFC-0146** (file-free until `#265`).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core` ledger / LLM-in-Core; Network profile семантика P0–P6; Book 0–IV.

> Примітка: драфти поза git — канон у репо = цей план + оновлений `desktop-ux.md` + QUEUE + RFC-0146 на закритті.

## 0. Навіщо (після N-fix)

N / N-fix зробили мережевий reference **чесним**. Desktop UI досі говорить мовою розробника (4 вкладки Work/Node/Network/Settings; `UNKNOWN→OFFLINE`; блокувальний submit у UI-потоці; немає офлайн F1).

```text
QUEUE N-fix closed (#248–#254 / RFC-0139)
  → Phase O: стабільний UX-контракт + Help каркас
    → не GPU marketplace; не новий monitoring daemon; не Core rewrite
```

## 1. Мета

1. Закріпити IA: **Робота / Стан системи / Параметри** + **Довідка · F1**.
2. Чесне представлення стану через `SystemSnapshot` (проєкція, не нова БД істини).
3. Неблокувальна робота GUI; refresh даних ≠ repaint.
4. Стабільні action / error / help ID + спільний словник з `Labels`.
5. Офлайн контекстна довідка без LLM.
6. Мова end-user на екрані; техніка — у подробицях.

Цільовий ярлик: **Desktop Developer Preview + honest end-user shell** (не production WCAG-сертифікація; не web UI).

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` **не** отримує GUI / Help / egui.
3. Немає залежності `aira-node → aira-desktop`.
4. C1 `Calculate 2 + 2` лишається `execution-basic`.
5. `DISCOVERED ≠ TRUSTED`; AddressBook ≠ живі сесії; UNKNOWN ≠ OFFLINE.
6. Help працює без інтернету / моделі / running node.
7. Не редагувати `Manifesto etc/`, `Meditation_About/`.
8. Не створювати універсальний GUI generator / окремий monitoring daemon / другу trust-систему.

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Required central help CDN
WCAG formal certification as merge gate
Rewriting Phase E–N history as never-DONE
Global live rendezvous beyond N-fix honesty (stays PARTIAL)
```

## 4. Смуги

```text
O0 Wiring (canon + QUEUE + living smoke)
  → O1 SystemSnapshot honesty
    → O2 Non-blocking submit/refresh
      → O3 Action/error/help IDs + Labels
        → O4 Shell IA (3 sections + Help chrome)
          → O5 Work screen
            → O6 System status screen
              → O7 Settings apply lifecycle
                → O8 Offline F1 shell
                  → O9 Seed help topics
                    → O10 RFC-0146 close
```

## 5. Атоми → QUEUE `#255`–`#265`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#255` | O0 | Phase O wiring | **DONE**: цей план IN PROGRESS; `desktop-ux.md` канон; QUEUE `#255`–`#265`; `phase_o_doc`; RFC-0146 reserved file-free | Snapshot (#256) |
| `#256` | O1 | SystemSnapshot honesty | **DONE** @ RFC-0147: typed projection; UNKNOWN≠OFFLINE; book≠sessions | non-blocking (#257) |
| `#257` | O2 | Non-blocking work/refresh | **DONE** @ RFC-0148: submit/refresh off egui `update()`; repaint≠data | IDs (#258) |
| `#258` | O3 | Action/error/help IDs | **DONE** @ RFC-0149: stable IDs + lexicon map | shell IA (#259) |
| `#259` | O4 | Shell IA | **DONE** @ RFC-0150: Work / System / Settings + Help·F1 chrome; docs | Work body (#260) |
| `#260` | O5 | Work screen | **DONE** @ RFC-0151: user-language; draft; Ctrl+Enter; provenance; tests | status screen (#261) |
| `#261` | O6 | System status screen | **DONE** @ RFC-0152: Program/Model/Connection/Events; honesty; tests | settings (#262) |
| `#262` | O7 | Settings apply lifecycle | Saved≠Applied≠Restart needed; tests | F1 shell (#263) |
| `#263` | O8 | Offline F1 shell | embed help; search; context routing; works offline; tests | topics (#264) |
| `#264` | O9 | Seed help topics | uk/en topics for catalog IDs; link check; tests | RFC close (#265) |
| `#265` | O10 | RFC-0146 + close | consolidating RFC-0146; QUEUE O closed | — |

```text
#255 O0 wiring DONE
  → #256 O1 snapshot DONE @ RFC-0147
    → #257 O2 non-blocking DONE @ RFC-0148
      → #258 O3 IDs DONE @ RFC-0149
        → #259 O4 shell DONE @ RFC-0150
          → #260 O5 work DONE @ RFC-0151
            → #261 O6 status DONE @ RFC-0152
              → #262 O7 settings OPEN
                → #263 O8 F1
                  → #264 O9 topics
                    → #265 O10 RFC-0146
```

## 6. Acceptance Phase O

```text
QUEUE `#255`–`#265` DONE; no OPEN O atoms.
desktop-ux.md IA matches running shell.
SystemSnapshot honesty: UNKNOWN≠OFFLINE; AddressBook≠sessions.
GUI does not block on HTTP submit/refresh in update().
F1 offline without node/LLM.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no ledger/GUI deps.
Анти-місія не порушена.
```

## 7. RFC

- Consolidating (`#265`): **AIRA-RFC-0146** — id **confirmed free** at `#255` (file-free until close).
- Per-atom: `#256`→[`AIRA-RFC-0147`](../specs/rfc/AIRA-RFC-0147-system-snapshot-honesty.md); `#257`→[`AIRA-RFC-0148`](../specs/rfc/AIRA-RFC-0148-nonblocking-work-refresh.md); `#258`→[`AIRA-RFC-0149`](../specs/rfc/AIRA-RFC-0149-action-error-help-ids.md); `#259`→[`AIRA-RFC-0150`](../specs/rfc/AIRA-RFC-0150-shell-ia-help-f1.md); `#260`→[`AIRA-RFC-0151`](../specs/rfc/AIRA-RFC-0151-work-screen-provenance.md); `#261`→[`AIRA-RFC-0152`](../specs/rfc/AIRA-RFC-0152-system-status-screen.md); further RFC-D у `#262`–`#264` за потреби.

## 8. Activation rule

```text
QUEUE N-fix closed @ RFC-0139 + developer «канон + Phase O» 2026-09-06
  → цей план активовано атомом `#255`.
`#255`–`#261` DONE; first OPEN `#262`.
```
