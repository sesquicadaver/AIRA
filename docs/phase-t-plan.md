# Phase T — Operation lifecycle honesty (post-S)

**Статус:** **IN PROGRESS** (активовано 2026-09-09; wiring `#306` **DONE**; `#307` **DONE** @ RFC-0193; `#308` **DONE** @ RFC-0194; first OPEN `#309`).  
**Джерела:** post-S audit `aira-current.md` @ `3301d27` (2026-09-09; merge PR #269 / RFC-0182); Phase S [`phase-s-plan.md`](phase-s-plan.md) **DONE** @ RFC-0182; **QUEUE S closed**; канон [`desktop-ux.md`](desktop-ux.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase T `#306`–`#312`.  
**Closure:** consolidating **RFC-0192** (id **confirmed free** 2026-09-09; file-free until `#312`).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core` ledger / LLM-in-Core; GUI stack (egui); IA Work / System / Settings + Help·F1; Claiming Polygon ledger DONE; Claiming global live mesh DONE; QUEUE S історію як never-DONE; public bind / auto-trust як Desktop default; admission hash policy (`check_activated`).

> Примітка: `aira-current.md` поза git-каноном (`p-only/`). Канон у репо = цей план + `desktop-ux.md` + QUEUE + RFC-D.

---

## 0. Навіщо (після S)

Phase S зміцнила **перевірку** (key-bound reachability, descriptor reopen, incomplete identity, submit∥lifecycle, observe streaming, durability honesty). Аудит @ `3301d27`: нові операції (opt-in dial, background observe) не доведені до повного життєвого циклу — **початок → виконання → успіх/помилка → очищення → достовірний GUI**.

Головні залишки:

1. Закрита TCP-сесія після dial показується як `live_session_count` до 5 хв (і після reopen Desktop).
2. Opt-in dial знову синхронний у UI-потоці (`block_on`) — блокує F1/навігацію.
3. Невдалий background observe губить mismatch → повторне повне хешування на кожен refresh.
4. Quit під час Submit лишає прихований `quit_after_stop` без завершення виходу.
5. Невдалий dial уже записав AddressBook перед мережевою спробою.

```text
QUEUE S closed (#295–#305 / RFC-0182)
  → Phase T IN PROGRESS: operation lifecycle honesty
    → не Core rewrite; не GPU marketplace; не public bind default
    → не новий scheduler / monitoring daemon
```

---

## 1. Мета

1. Dial evidence = **історія handshake**, не жива сесія; `live_session_count` не з історичного файла.
2. Opt-in dial виконується **поза UI-потоком** (існуючий jobs/slot патерн); F1/навігація доступні під час спроби.
3. Background observe має явний результат `Pending` / `Ready` / `Failed` для версії pointer/cache; fail не rehash на кожен refresh без зміни входу.
4. Quit під час Submit: або відкладений Stop→Close після Submit з явним повідомленням, або reject без прихованого прапорця (обрати й зафіксувати один контракт).
5. Невдалий пробний dial **не** тихо замінює authoritative AddressBook (rollback або окремий «зберегти адресу»).

Цільовий ярлик: **Desktop Developer Preview + operation lifecycle honesty** (не WCAG; не full global mesh DONE).

---

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR (після активації).
2. `aira-core` **не** отримує GUI / Help / egui / ledger impl.
3. IA Phase O **не** переписується на 4 вкладки.
4. C1 `Calculate 2 + 2` → `execution-basic`.
5. `DISCOVERED ≠ TRUSTED`; AddressBook ≠ живі сесії; UNKNOWN ≠ OFFLINE.
6. Saved ≠ Applied; Applied лише з confirmed runtime.
7. Help офлайн без LLM.
8. Не редагувати `Manifesto etc/`, `Meditation_About/`.
9. Admission/activation/reachability evidence **не** послаблюється заради UX/швидкості.
10. **Не** робити public bind / auto-trust default Desktop.
11. AddressBook лишається dial authority (змінюємо лише момент/умови запису).

---

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Нова вкладка Network
Public bind / auto-trust як default
Claiming global live mesh DONE
Rewriting Phase O–S as never-DONE
Новий великий scheduler / orchestration / monitoring daemon
Full crash-safe multi-file DB
Cosmetic theme-only PRs
Weakening observe admission hash
```

---

## 4. Смуги

```text
T0 Wiring (план у QUEUE + living smoke; RFC-0192 reserved)
  → T1 Dial evidence ≠ live session
    → T1 Opt-in dial off UI-thread
      → T1 Observe miss fail durable
        → T2 Quit∥Submit completion
          → T2 Dial AddressBook candidate honesty
            → T close RFC-0192
```

---

## 5. Атоми → QUEUE `#306`–`#312`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#306` | T0 | Phase T wiring | план IN PROGRESS; QUEUE `#306`–`#312`; `phase_t_doc`; tips; RFC-0192 reserved file-free | dial live (#307) |
| `#307` | T1 | Dial evidence ≠ live session | **DONE** @ RFC-0193: evidence ≠ live count; last handshake only | dial async (#308) |
| `#308` | T1 | Opt-in dial off UI-thread | **DONE** @ RFC-0194: dial через jobs/slot; UI не `block_on`; F1/nav | observe fail (#309) |
| `#309` | T1 | Observe miss fail durable | Pending/Ready/Failed на версію; fail без rehash storm; admit не послаблюється | Quit∥Submit (#310) |
| `#310` | T2 | Quit∥Submit completion | один контракт: deferred Stop→Close **або** reject без sticky flag; тест сценарію | AddressBook (#311) |
| `#311` | T2 | Dial AddressBook candidate honesty | невдалий dial не тихо overwrite authority; rollback **або** explicit save | RFC close (#312) |
| `#312` | T | RFC-0192 + close | consolidating RFC-0192; QUEUE T closed; no OPEN T atoms | — |

```text
#306 T0 wiring (DONE this atom)
  → #307 T1 dial evidence ≠ live DONE @ RFC-0193
    → #308 T1 dial off-UI DONE @ RFC-0194
      → #309 T1 observe fail durable (first OPEN)
        → #310 T2 Quit∥Submit
          → #311 T2 AddressBook candidate
            → #312 T close RFC-0192
```

**Anti-merge:** не зливати dial live-honesty з AddressBook rollback; не зливати observe fail-cache з dial async; не зливати Quit∥Submit з dial atoms.

---

## 6. Acceptance Phase T (після активації й close)

```text
QUEUE `#306`–`#312` DONE; no OPEN T atoms.
Успішний dial → закритий сокет → snapshot: є історія handshake, немає вигаданої живої сесії.
Dial під час недоступного endpoint: F1 і навігація працюють.
Пошкоджені weights → worker fail → кілька refresh: стійка помилка; немає безперервного rehash.
Submit → Quit → кінець Submit: вихід передбачуваний (без sticky hidden flag).
Відома адреса A → невдалий пробний dial B: book зберігає A або явна погоджена зміна.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no GUI/ledger deps.
Анти-місія не порушена; public bind не став default.
```

---

## 7. RFC (після активації)

- Consolidating (`#312`): **AIRA-RFC-0192** — id **confirmed free** 2026-09-09 (немає `AIRA-RFC-0192*` у дереві; file-free until close).
- Per-atom RFC-D: `#307`→**0193** … `#311`→**0197** (призначаються на кожному PR); `#312`→0192 consolidating.

---

## 8. Activation rule

```text
QUEUE S closed @ RFC-0182
  + developer «активувати Phase T» (2026-09-09)
  → wiring `#306` DONE; план IN PROGRESS; first OPEN `#307`.
```

---

## 9. Матриця evidence → атоми (з `aira-current.md` @ `3301d27`)

| Аудит § | Критерій | Атом |
|---------|----------|------|
| §2 P1 | dial evidence → fake live session | `#307` |
| §3 P1 | dial blocks UI / F1 | `#308` |
| §4 P1 | observe fail lost / rehash loop | `#309` |
| §5 P2 | Quit∥Submit sticky flag | `#310` |
| §6 P2 | failed dial mutates AddressBook | `#311` |
| §7–8 | non-goals / test matrix | plan + close `#312` |
