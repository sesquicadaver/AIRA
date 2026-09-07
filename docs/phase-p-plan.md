# Phase P — Runtime-honest Desktop (post-O)

**Статус:** складено 2026-09-07; **DONE** @ [`AIRA-RFC-0156`](../specs/rfc/AIRA-RFC-0156-phase-p-runtime-honest-desktop.md) — QUEUE `#266`–`#274` **DONE**; **QUEUE P closed**; no OPEN P atoms.  
**Джерела:** post-O audit `aira-current.md` @ `700ffd5` (2026-09-07); Phase O [`phase-o-plan.md`](phase-o-plan.md) **DONE** @ RFC-0146; канон [`desktop-ux.md`](desktop-ux.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase P `#266`–`#274`.  
**Closure:** consolidating **RFC-0156** (id **confirmed free** at `#266`; file at `#274`).  
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
| `#268` | P1 | Applied from runtime | **DONE** @ RFC-0158: confirmed runtime/attach; Undefined when unconfirmed; status sync | model (#269) |
| `#269` | P1 | Model triple UX | **DONE** @ RFC-0159: selected≠ready≠used; strip+System; Settings observe-only | reachability (#270) |
| `#270` | P1 | Reachability endpoint bind | **DONE** @ RFC-0160: endpoint+direction; CLI evidence ≠ transcript-alone | EVM honesty (#271) |
| `#271` | P1 | EVM/JSON-RPC honesty | **DONE** @ RFC-0161: https config; Mock≠ledger; PARTIAL | lifecycle (#272) |
| `#272` | P2 | Lifecycle non-blocking | **DONE** @ RFC-0162: Start/Stop/Quit off update(); refresh invalidate | Help (#273) |
| `#273` | P2 | Help F1 routing | **DONE** @ RFC-0163: clear search; focus→section→screen; localized titles | RFC close (#274) |
| `#274` | P | RFC-0156 + close | **DONE** @ RFC-0156; QUEUE P closed; no OPEN P atoms | — |

```text
#266 P0 wiring DONE
  → #267 P1 freshness DONE @ RFC-0157
    → #268 P1 applied-runtime DONE @ RFC-0158
      → #269 P1 model triple DONE @ RFC-0159
        → #270 P1 reachability endpoint DONE @ RFC-0160
          → #271 P1 EVM honesty DONE @ RFC-0161
            → #272 P2 lifecycle jobs DONE @ RFC-0162
              → #273 P2 Help routing DONE @ RFC-0163
              → #274 RFC-0156 close DONE
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

- Consolidating (`#274`): **AIRA-RFC-0156** — id **confirmed free** at `#266`; file at close.
- Per-atom RFC-D: `#267`→**0157** … `#272`→**0162** (DONE) … `#273`→**0163** (DONE) … `#274`→0156 consolidating.

## 8. Activation rule

```text
QUEUE O closed @ RFC-0146 + developer «скласти Phase P» 2026-09-07
  → цей план активовано атомом `#266`.
`#266`–`#274` DONE @ RFC-0156; **QUEUE P closed**; no OPEN P atoms.
```

## 9. Після Phase P

Наступний етап: [`phase-q-plan.md`](phase-q-plan.md) (Phase Q; QUEUE `#275`–`#285`).
