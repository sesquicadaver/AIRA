# Phase P — Runtime-honest Desktop (post-O)

**Статус:** складено 2026-09-07; **IN PROGRESS** — QUEUE `#266`–`#267` **DONE** @ RFC-0157; first OPEN `#268`.  
**Джерела:** post-O audit `aira-current.md` @ `700ffd5` (2026-09-07); Phase O [`phase-o-plan.md`](phase-o-plan.md) **DONE** @ RFC-0146; канон [`desktop-ux.md`](desktop-ux.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase P `#266`–`#274`.  
**Closure:** consolidating **RFC-0156** (id **confirmed free** at `#266`; file-free until `#274`).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core` ledger / LLM-in-Core; GUI stack (egui); IA Work/System/Settings + Help·F1; QUEUE O історію.

> Примітка: `aira-current.md` / `aira-gui.md` поза git-каноном. Канон у репо = цей план + `desktop-ux.md` + QUEUE + RFC-D.

## 0. Навіщо (після O)

Phase O дала **стабільний каркас** (3 розділи, F1, jobs, lexicon, provenance). Аудит @ `700ffd5`: частина індикаторів досі показує **конфіг / наявність файла / identity** як підтверджений поточний стан.

```text
QUEUE O closed (#255–#265 / RFC-0146)
  → Phase P: підключити shell до перевірених runtime-фактів
    → не нові вкладки; не GPU marketplace; не Core rewrite
```

## 1. Мета

1. **Назва стану = перевірений факт**, не наявність поля/файла/налаштування.
2. Модель: **вибрана ≠ готова ≠ використана в результаті**.
3. Settings **Applied** з підтвердженого runtime, не з копії settings при старті GUI.
4. Snapshot freshness: `Current` / `Stale` / `Unavailable` за часом вимірювання й процесами.
5. Reachability attestation прив’язана до **endpoint + напрямку** сесії.
6. EVM/JSON-RPC: чесний PARTIAL (не видавати Mock HTTP за ledger).
7. Start/Stop не блокують F1; Help routing не ламається старим пошуком.

Цільовий ярлик: **Desktop Developer Preview + runtime-honest monitoring** (не WCAG-сертифікація; не web UI).

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` **не** отримує GUI / Help / egui / ledger impl.
3. IA Phase O **не** переписується (Work / System / Settings + Help·F1).
4. C1 `Calculate 2 + 2` лишається `execution-basic`.
5. `DISCOVERED ≠ TRUSTED`; AddressBook ≠ живі сесії; UNKNOWN ≠ OFFLINE.
6. Help офлайн без LLM; ID тем стабільні.
7. Не редагувати `Manifesto etc/`, `Meditation_About/`.
8. Не створювати monitoring daemon / універсальний GUI generator / другу trust-систему.

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Required central help CDN
Full Polygon mainnet as CI default
Claiming global live mesh DONE
Rewriting Phase O as never-DONE
Cosmetic theme-only PRs without honesty fixes
```

## 4. Смуги

```text
P0 Wiring
  → P1 Snapshot freshness
    → P1 Applied-from-runtime
      → P1 Model triple (selected/ready/used)
        → P1 Reachability endpoint bind
          → P1 EVM/JSON-RPC honesty
            → P2 Lifecycle non-blocking
              → P2 Help F1 routing
                → P close RFC-0156
```

## 5. Атоми → QUEUE `#266`–`#274`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#266` | P0 | Phase P wiring | **DONE**: цей план IN PROGRESS; QUEUE `#266`–`#274`; `phase_p_doc`; `desktop-ux` tip; RFC-0156 reserved file-free | freshness (#267) |
| `#267` | P1 | Snapshot freshness | **DONE** @ RFC-0157: `DataQuality::Stale`; measurement≠load; bind≠listener proof | applied (#268) |
| `#268` | P1 | Applied from runtime | applied values from confirmed runtime/attach; Undefined when unconfirmed; reopen/submit-start sync; tests + RFC-D | model (#269) |
| `#269` | P1 | Model triple UX | selected / ready / used-in-result separate; strip+System; Settings not placeholder-as-done; tests + RFC-D | reachability (#270) |
| `#270` | P1 | Reachability endpoint bind | attestation binds endpoint+direction; CLI transcript-alone ≠ verified; tests + RFC-D | EVM honesty (#271) |
| `#271` | P1 | EVM/JSON-RPC honesty | https config path for declared Amoy/mainnet; Mock HTTP ≠ ledger claim; docs PARTIAL; tests + RFC-D | lifecycle (#272) |
| `#272` | P2 | Lifecycle non-blocking | Start/Stop/Quit off egui `update()`; invalidate refresh generation; tests + RFC-D | Help (#273) |
| `#273` | P2 | Help F1 routing | clear search on contextual open; focus→section→screen; localized titles; tests + RFC-D | RFC close (#274) |
| `#274` | P | RFC-0156 + close | consolidating RFC-0156; QUEUE P closed | — |

```text
#266 P0 wiring DONE
  → #267 P1 freshness DONE @ RFC-0157
    → #268 P1 applied-runtime OPEN
      → #269 P1 model triple
        → #270 P1 reachability endpoint
          → #271 P1 EVM honesty
            → #272 P2 lifecycle jobs
              → #273 P2 Help routing
                → #274 RFC-0156 close
```

## 6. Acceptance Phase P

```text
QUEUE `#266`–`#274` DONE; no OPEN P atoms.
Stale/unavailable cannot paint as Current connection.
Applied reflects confirmed runtime or Explicitly Undefined.
Model strip/System show selected≠ready≠used.
Reachability attestation requires bound endpoint+direction.
JSON-RPC Mock path not documented/tested as ledger success.
Start/Stop do not block F1/navigation in update().
F1 contextual open not overridden by stale search.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no ledger/GUI deps.
Анти-місія не порушена.
```

## 7. RFC

- Consolidating (`#274`): **AIRA-RFC-0156** — id **confirmed free** at `#266` (file-free until close).
- Per-atom RFC-D: `#267`→**0157** (DONE) … `#273`→0163 (assigned at each atom PR).

## 8. Activation rule

```text
QUEUE O closed @ RFC-0146 + developer «скласти Phase P» 2026-09-07
  → цей план активовано атомом `#266`.
`#266`–`#267` DONE @ RFC-0157; first OPEN `#268`.
```