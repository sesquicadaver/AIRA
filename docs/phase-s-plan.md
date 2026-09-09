# Phase S — Cross-boundary integrity & connect honesty (post-R)

**Статус:** **IN PROGRESS** (активовано 2026-09-09; wiring `#295` **DONE**; first OPEN `#296`).  
**Джерела:** post-R audit `aira-current.md` @ `e5aee86` (2026-09-09; merge PR #257 / RFC-0174); Phase R [`phase-r-plan.md`](phase-r-plan.md) **DONE** @ RFC-0174; QUEUE R closed; Phase Q [`phase-q-plan.md`](phase-q-plan.md) **DONE** @ RFC-0164; канон [`desktop-ux.md`](desktop-ux.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase S `#295`–`#305`.  
**Closure:** consolidating **RFC-0182** (id **confirmed free** 2026-09-09; file-free until `#305`).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core` ledger / LLM-in-Core; GUI stack (egui); IA Work / System / Settings + Help·F1; Claiming Polygon ledger DONE; Claiming global live mesh DONE; QUEUE R історію як never-DONE; public bind / auto-trust як Desktop default.

> Примітка: `aira-current.md` поза git-каноном (`p-only/`). Канон у репо = цей план + `desktop-ux.md` + QUEUE + RFC-D.

---

## 0. Навіщо (після R)

Phase Q/R закрили **локальні** honesty/UX-розриви: install-scoped identity, observe-cache, lifecycle↔refresh, Connection CTA/controls/Help. Аудит @ `e5aee86`: черга DONE ≠ доказ готовності на **межах** (новий процес, інший ключ під тим самим ID, інший комп’ютер, перерваний запис, команда під час іншої команди).

Головні залишки:

1. Reachability admission звіряє **ім’я** identity, а підпис — з ключа всередині імпортованого набору.
2. Root-scoped verify моделі не покриває зовнішній `ArtifactDescriptor` після reopen.
3. Неповна пара identity-файлів тихо мінтить нову identity.
4. Connect-сценарій Phase R завершує **налаштування**, не peer-сесію; Help може завищувати очікування (loopback ≠ адреса для іншого ПК).
5. CTA після Stop може вести в Refresh замість Start; submit∥Start/Stop; observe cache-miss знову важкий у UI-потоці; replay/state без спільної commit-межі.

```text
QUEUE R closed (#286–#294 / RFC-0174)
  → Phase S IN PROGRESS: cross-boundary integrity + connect honesty
    → не Core rewrite; не GPU marketplace; не public bind default
```

---

## 1. Мета

1. Імпортований reachability evidence **відхиляється**, якщо `target_identity_ref` збігається, але ключ ≠ authoritative root-scoped local key.
2. Node-signed model **descriptor + payload** успішно verify після cold reopen **без** priming глобального process keyring.
3. Втрата одного з identity-файлів → **fail-closed** стан пошкодження; не автогенерація нової identity.
4. Документований Desktop connect-шлях або (а) завершується підтвердженою автентифікованою peer-сесією за явною дозволеною адресою, або (б) Help/CTA **точно** описують межу (setup ≠ remote connect; loopback ≠ dial target).
5. Connection CTA після Stop пропонує Start, коли runtime зупинено й мережевий профіль потребує apply/observe — без тупикового Refresh-only.
6. Submit / Start / Stop мають взаємне виключення або визначену чергу допуску.
7. Observe cache-miss не блокує UI повним `fs::read` weights; hash — потоковий; admission перед execute не послаблюється.
8. Reachability replay+state: або crash-safer commit boundary, або чесна документація гарантії (не «атомарно» ширше за факт).

Цільовий ярлик: **Desktop Developer Preview + cross-boundary integrity** (не WCAG; не full global mesh DONE).

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

---

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Нова вкладка Network
Public bind / auto-trust як default
Claiming global live mesh DONE
Rewriting Phase O–R as never-DONE
Silent identity migration / rewrite existing install IDs
Full crash-safe multi-file DB для всього Desktop state (лише scoped reachability/invite guarantees)
Новий великий scheduler / orchestration framework
Cosmetic theme-only PRs
```

---

## 4. Смуги

```text
S0 Wiring (план у QUEUE + living smoke; RFC-0182 reserved)
  → S1 Reachability key-bound admission
    → S1 Model descriptor root-scoped verify (reopen)
      → S1 Identity incomplete-pair fail-closed
        → S1 Connect Help boundary honesty
          → S1 Opt-in peer dial / session evidence (явна адреса)
            → S2 CTA Stop→Start matrix
              → S2 Submit∥Start/Stop admission
                → S2 Observe miss off UI-thread
                  → S2 Reachability durability honesty/commit
                    → S close RFC-0182
```

---

## 5. Атоми → QUEUE `#295`–`#305`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#295` | S0 | Phase S wiring | **DONE**: план IN PROGRESS; QUEUE `#295`–`#305`; `phase_s_doc`; tips; RFC-0182 reserved file-free | reach key (#296) |
| `#296` | S1 | Reachability key-bound admission | reject same-ID / foreign-key import; verify vs root keyring; negative test | model reopen (#297) |
| `#297` | S1 | Model descriptor root-scoped verify | descriptor+payload verify після cold reopen без global keyring priming; тест | identity pair (#298) |
| `#298` | S1 | Identity incomplete-pair fail-closed | mint лише якщо **обох** файлів немає; partial → error; secret perms fail-closed | Help boundary (#299) |
| `#299` | S1 | Connect Help boundary honesty | Help/CTA EN+UK: setup ≠ remote session; loopback ≠ peer dial address | dial path (#300) |
| `#300` | S1 | Opt-in peer dial / session evidence | явна дозволена адреса + dial + confirmed handshake evidence у Desktop path (без public default) | CTA Stop (#301) |
| `#301` | S2 | CTA Stop→Start matrix | stopped runtime → Start (не Refresh-only) без фейкового CONNECTED; transition tests | submit gate (#302) |
| `#302` | S2 | Submit∥Start/Stop admission | взаємне виключення / черга; немає подвійного start() | observe UI (#303) |
| `#303` | S2 | Observe miss off UI-thread | cache miss → background/streaming hash; UI не `fs::read` повних weights; admit не послаблюється | durability (#304) |
| `#304` | S2 | Reachability durability honesty | crash-safer replay+state **або** docs обмежують «атомарно» до реальної гарантії | RFC close (#305) |
| `#305` | S | RFC-0182 + close | consolidating RFC-0182; QUEUE S closed; no OPEN S atoms | — |

```text
#295 S0 wiring DONE
  → #296 S1 reachability key-bound (first OPEN)
    → #297 S1 model descriptor reopen
      → #298 S1 identity incomplete-pair
        → #299 S1 Help connect boundary
          → #300 S1 opt-in dial/session
            → #301 S2 CTA Stop→Start
              → #302 S2 submit∥lifecycle
                → #303 S2 observe off-UI
                  → #304 S2 durability honesty
                    → #305 S close RFC-0182
```

**Anti-merge:** не зливати key-bound admission з dial UX; не зливати Help-boundary з dial implementation; не зливати observe UI з admission hash policy.

---

## 6. Acceptance Phase S (після активації й close)

```text
QUEUE `#295`–`#305` DONE; no OPEN S atoms.
Same-ID / foreign-key reachability import → reject; state unchanged.
Model evidence (descriptor+payload) verify після cold reopen без process keyring priming.
Partial identity files → fail-closed; no silent new ID.
Help connect не обіцяє remote session лише після invite+P1+Stop/Start на loopback.
Opt-in dial path дає підтверджену сесію **або** атом `#300` WONT-NEED з явною документованою межею (лише якщо `#299` уже достатньо і розробник підтвердить).
CTA після Stop веде на Start, коли це наступний крок.
Submit не стартує паралельно з in-flight lifecycle Start.
Observe miss не блокує UI повним read weights.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no GUI/ledger deps.
Анти-місія не порушена; public bind не став default.
```

---

## 7. RFC (після активації)

- Consolidating (`#305`): **AIRA-RFC-0182** — id **confirmed free** 2026-09-09 (немає `AIRA-RFC-0182*` у дереві; file-free until close).
- Per-atom RFC-D: `#296`→**0183** … `#304`→**0191** (призначаються на кожному PR); `#305`→0182 consolidating.

---

## 8. Activation rule

```text
QUEUE R closed @ RFC-0174
  + developer «активувати Phase S» (2026-09-09)
  → wiring `#295` DONE; план IN PROGRESS; first OPEN `#296`.
```

---

## 9. Матриця evidence → атоми (з `aira-current.md` @ `e5aee86`)

| Аудит § | Критерій | Атом |
|---------|----------|------|
| §2 HIGH | reachability key-bound | `#296` |
| §3 HIGH | descriptor reopen | `#297` |
| §4 HIGH | incomplete identity pair | `#298` |
| §5 HIGH | connect scenario honesty / session | `#299` + `#300` |
| §6 MEDIUM | CTA after Stop | `#301` |
| §7 MEDIUM | submit∥lifecycle | `#302` |
| §8 MEDIUM | observe miss UI | `#303` |
| §9 | durability / «атомарно» honesty | `#304` |
