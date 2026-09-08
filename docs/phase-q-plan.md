# Phase Q — Cross-install integrity & light monitoring (post-P)

**Статус:** складено 2026-09-07; **IN PROGRESS** — QUEUE `#275`–`#284` **DONE** @ RFC-0173; first OPEN `#285`.  
**Джерела:** post-P audit `aira-current.md` @ `00f19cf` (2026-09-07); Phase P [`phase-p-plan.md`](phase-p-plan.md) **DONE** @ RFC-0156; QUEUE P closed; канон [`desktop-ux.md`](desktop-ux.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase Q `#275`–`#285`.  
**Closure:** consolidating **RFC-0164** (id **confirmed free** at `#275`; file-free until `#285`).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core` ledger / LLM-in-Core; GUI stack (egui); IA Work/System/Settings + Help·F1; Claiming Polygon ledger DONE; Claiming global live mesh DONE; QUEUE P історію як never-DONE.

> Примітка: `aira-current.md` поза git-каноном. Канон у репо = цей план + `desktop-ux.md` + QUEUE + RFC-D.

## 0. Навіщо (після P)

Phase P закрила **чергу honesty-лейблів** (freshness, Applied, model triple, reachability bind, EVM PARTIAL, non-blocking lifecycle, F1 routing). Аудит @ `00f19cf`: черга DONE ≠ доказ готовності до звичайної експлуатації. Головні розриви — **наскрізні**: дві чисті інсталяції, reopen identity/keyring, важкий model observe, reachability freshness/NAT/evidence admission, lifecycle races.

```text
QUEUE P closed (#266–#274 / RFC-0156)
  → Phase Q: production-path integrity на стандартному Desktop
    → не Core rewrite; не GPU marketplace; не нові вкладки
```

## 1. Мета

1. Повний зелений CI на tip після fmt/регресій (не лише DONE-рядки QUEUE).
2. Дві незалежні Desktop-інсталяції → **різні** постійні identity ID; без тихого overwrite чужого ключа.
3. Легкий моніторинг **без** повного `fs::read`+hash weights кожні ~2 с; admission hash лишається окремою операцією.
4. Ready/verify після reopen з **root-scoped** verification context (не випадковий process keyring).
5. Reachability: локальний bind **не** продовжує зовнішній DIRECT; dial≠accepted NAT; evidence прив’язаний до root + replay/apply-time.
6. Одна lifecycle/config ревізія на операцію; Quit під час Start коректний; Applied з worker snapshot.
7. `used` модель ≠ backend id; invite import атомарний (або явний partial).

Цільовий ярлик: **Desktop Developer Preview + cross-install / light-monitor integrity** (не WCAG; не web UI; не full global mesh).

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` **не** отримує GUI / Help / egui / ledger impl.
3. IA Phase O **не** переписується.
4. C1 `Calculate 2 + 2` лишається `execution-basic`.
5. `DISCOVERED ≠ TRUSTED`; AddressBook ≠ живі сесії; UNKNOWN ≠ OFFLINE.
6. Help офлайн без LLM.
7. Не редагувати `Manifesto etc/`, `Meditation_About/`.
8. Не створювати monitoring daemon / другу trust-систему / універсальний GUI generator.
9. Admission/activation evidence **не** послаблюється заради швидкості observe.

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Full Polygon mainnet as CI default
Claiming global live mesh DONE
Rewriting Phase P as never-DONE
Cosmetic theme-only PRs
Silent identity migration that rewrites existing install IDs
```

## 4. Смуги

```text
Q0 Wiring (+ CI fmt)
  → Q1 Unique Desktop identity
    → Q1 Model light observe
      → Q1 Model verify context (reopen)
        → Q1 Reachability observation independence
          → Q1 Reachability NAT endpoints
            → Q1 Reachability evidence admission
              → Q2 Lifecycle revision races
                → Q2 Backend≠model used
                  → Q2 Invite atomic import
                    → Q close RFC-0164
```

## 5. Атоми → QUEUE `#275`–`#285`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#275` | Q0 | Phase Q wiring | цей план IN PROGRESS; QUEUE `#275`–`#285`; `phase_q_doc`; tips; RFC-0164 reserved file-free; **`cargo fmt` зелений на tip** | identity (#276) |
| `#276` | Q1 | Unique Desktop identity | **DONE** @ RFC-0165: install-scoped ID; TrustStore KeyCollision; two-root bootstrap | model observe (#277) |
| `#277` | Q1 | Model light observe | **DONE** @ RFC-0166: light observe + observe-ready cache; admit full-hash | verify context (#278) |
| `#278` | Q1 | Model verify context | **DONE** @ RFC-0167: root-scoped verify; reopen without test keyring | reach obs (#279) |
| `#279` | Q1 | Reachability observation independence | **DONE** @ RFC-0168: окремі clocks; bind ≠ external freshness; skew bound | NAT (#280) |
| `#280` | Q1 | Reachability NAT endpoints | **DONE** @ RFC-0169: dial≠accept local_addr; inbound NAT not fail-closed | evidence (#281) |
| `#281` | Q1 | Reachability evidence admission | **DONE** @ RFC-0170: root-bound apply; durable replay; apply-time freshness | lifecycle (#282) |
| `#282` | Q2 | Lifecycle revision races | **DONE** @ RFC-0171: one revision; refresh gate; Quit→Stop; Applied from worker snapshot | provenance (#283) |
| `#283` | Q2 | Backend≠model used | **DONE** @ RFC-0172: used = model id/hash only; no `backend:*` without evidence | invite (#284) |
| `#284` | Q2 | Invite import atomicity | **DONE** @ RFC-0173: checks before writes; trust rollback on book fail | RFC close (#285) |
| `#285` | Q | RFC-0164 + close | consolidating RFC-0164; QUEUE Q closed; no OPEN Q atoms; doc tip drift cleared | — |

```text
#275 Q0 wiring (+ fmt)
  → #276 Q1 unique identity DONE @ RFC-0165
    → #277 Q1 model light observe DONE @ RFC-0166
      → #278 Q1 model verify context DONE @ RFC-0167
        → #279 Q1 reach observation independence DONE @ RFC-0168
          → #280 Q1 reach NAT endpoints DONE @ RFC-0169
            → #281 Q1 reach evidence admission DONE @ RFC-0170
              → #282 Q2 lifecycle revision DONE @ RFC-0171
                → #283 Q2 backend≠model DONE @ RFC-0172
                  → #284 Q2 invite atomicity DONE @ RFC-0173
                    → #285 RFC-0164 close
```

## 6. Acceptance Phase Q

```text
QUEUE `#275`–`#285` DONE; no OPEN Q atoms.
CI fmt+required checks green on tip.
Two clean Desktop roots → distinct identity IDs; no silent TrustStore key clobber.
Status refresh does not fully hash model weights every interval.
Reopen Desktop with activated model shows ready without test-only keyring priming.
Local bind does not refresh external DIRECT as Current.
NAT dial≠local accepted endpoints can produce valid local evidence.
Evidence apply is root-bound, non-replayable, apply-time fresh.
Lifecycle Quit/Start/refresh/applied races closed per Done when.
used model field is not a backend id.
Invite failure does not leave silent TrustStore mutation.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no ledger/GUI deps.
Анти-місія не порушена.
```

## 7. RFC

- Consolidating (`#285`): **AIRA-RFC-0164** — id **confirmed free** at `#275` (file-free until close).
- Per-atom RFC-D: `#276`→**0165** (DONE); `#277`→**0166** (DONE); `#278`→**0167** (DONE); `#279`→**0168** (DONE); `#280`→**0169** (DONE); `#281`→**0170** (DONE); `#282`→**0171** (DONE); `#283`→**0172** (DONE); `#284`→**0173** (DONE); `#285`→0164 consolidating.

## 8. Activation rule

```text
QUEUE P closed @ RFC-0156 + developer «скласти Phase Q» 2026-09-07
  → цей план активовано атомом `#275`.
`#275`–`#284` DONE @ RFC-0173; first OPEN `#285`.
```
