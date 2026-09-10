# Phase U — Cross-path contract honesty (post-T)

**Статус:** **IN PROGRESS** (активовано 2026-09-10; wiring `#313` **DONE**; `#314` **DONE** @ RFC-0199; `#315` **DONE** @ RFC-0200; `#316` **DONE** @ RFC-0201; `#317` **DONE** @ RFC-0202; `#318` **DONE** @ RFC-0203; `#319` **DONE** @ RFC-0204; `#320` **DONE** @ RFC-0205; first OPEN `#321`; RFC-0198 reserved file-free).  
**Джерела:** post-T audit `aira-current.md` @ `ef5f69c` (2026-09-10; merge PR #276 / RFC-0192); Phase T [`phase-t-plan.md`](phase-t-plan.md) **DONE** @ RFC-0192; **QUEUE T closed**; канон [`desktop-ux.md`](desktop-ux.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase U `#313`–`#322`.  
**Closure:** consolidating **RFC-0198** (id **confirmed free** 2026-09-10; file-free until `#322`).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core` ledger / LLM-in-Core; GUI stack (egui); IA Work / System / Settings + Help·F1; Claiming Polygon ledger DONE; Claiming global live mesh DONE; QUEUE T історію як never-DONE; public bind / auto-trust як Desktop default; admission hash policy (`check_activated`).

> Примітка: `aira-current.md` поза git-каноном (`p-only/`). Канон у репо = цей план + `desktop-ux.md` + QUEUE + RFC-D.

---

## 0. Навіщо (після T)

Phase T замкнула **операційний lifecycle honesty** Desktop (dial evidence ≠ live, dial off-UI, observe fail sticky, Quit∥Submit, AddressBook candidate rollback). Аудит @ `ef5f69c`: основні Desktop-зауваження T зняті, але **базові шляхи результату / identity / audit / persistence** мають різні гарантії залежно від входу (CLI vs HTTP vs Desktop vs index).

Головні залишки (P1):

1. Verification бере `expression`/`action` з output виконавця → підмінена узгоджена пара проходить.
2. `get_result(problem_id)` може повернути `ProblemRecord.result` з mutable index без ArtifactStore verify.
3. CLI `identity create` перезаписує чинну пару; Desktop-захист не спільний.
4. PolicyGate `seq`/ID колізії + silent skip на persistence → втрата audit між submit.
5. Failed submit виходить до `persist_after_submit` → failure history не durable після reopen.

```text
QUEUE T closed (#306–#312 / RFC-0192)
  → Phase U IN PROGRESS: cross-path contract honesty
    → не Core rewrite; не GPU marketplace; не public bind default
    → не повний SQLite operational rewrite; не новий scheduler daemon
```

---

## 1. Мета

1. Verification перевіряє **доручену capsule** (action/expression), а не підмінений output; узгоджена підміна → `VerificationFailed`, без VRA.
2. Result-by-`problem_id` і result-by-artifact мають **однакові** гарантії цілісності (index = locator).
3. CLI і Desktop identity create — **одна** fail-closed операція; повторний create не змінює чинний secret/descriptor.
4. Кожне policy-рішення має унікальний ID; same-ID/different-hash на persistence = конфлікт, не silent skip.
5. Після прийняття submit durable запис завершується і для success, і для failure (problem state + failure history після reopen).
6. Штатний submit **чесно** відображає mock vs configured executor (без вигаданого «модель готова виконувати» лише з activated pointer).
7. AddressBook rollback відновлює **лише власну** candidate-зміну (не повний snapshot поверх паралельних writers).
8. Виконувані docs (systemd peer listen) узгоджені з prime-port validation.

Цільовий ярлик: **Reference v0.3-strict + cross-path contract honesty** (не production distributed; не WCAG; не full global mesh DONE).

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
11. ArtifactStore / canonical signatures лишаються authoritative для immutable результатів.
12. Не маскувати mock-execution як verified LLM output.

---

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Нова вкладка Network
Public bind / auto-trust як default
Claiming global live mesh DONE
Rewriting Phase O–T as never-DONE
Повний SQLite-backed operational plane rewrite
Новий великий scheduler / orchestration / monitoring daemon
Full O(N²)→SQLite event migration як єдиний атом із correctness
Cosmetic theme-only PRs
Weakening observe admission hash
Повний F1 non-technical model marketplace UX
```

**Відкладено (не U-атоми; наступна фаза / окремий план):** HTTP mutex→worker redesign; ProcessBackend full-tree deadline; CAS open без full recovery scan; Timestamp/SQLite token races (§10 аудиту) як окремий hardening-пакет; README «історія vs поточна поведінка» великий docs-split.

---

## 4. Смуги

```text
U0 Wiring (план у QUEUE + living smoke; RFC-0198 reserved)
  → U1 Verification capsule-sourced
    → U1 Result-by-problem ArtifactStore authority
      → U1 CLI identity create fail-closed
        → U1 Policy audit ID + persist conflict
          → U1 Failed submit durable
            → U2 Submit executor honesty (mock vs configured)
              → U2 AddressBook selective rollback
                → U2 systemd/docs prime-port
                  → U close RFC-0198
```

---

## 5. Атоми → QUEUE `#313`–`#322`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#313` | U0 | Phase U wiring | план IN PROGRESS; QUEUE `#313`–`#322`; `phase_u_doc`; tips; RFC-0198 reserved file-free | verify (#314) |
| `#314` | U1 | Verification capsule-sourced | **DONE** @ RFC-0199: capsule-sourced; substituted output fails | result index (#315) |
| `#315` | U1 | Result-by-problem authority | **DONE** @ RFC-0200: problem_id → ArtifactStore; index locator | identity (#316) |
| `#316` | U1 | CLI identity create fail-closed | **DONE** @ RFC-0201: shared create; no overwrite | policy IDs (#317) |
| `#317` | U1 | Policy audit uniqueness | **DONE** @ RFC-0202: unique IDs; same-ID≠hash conflict | failed submit (#318) |
| `#318` | U1 | Failed submit durable | **DONE** @ RFC-0203: Err після accept → durable failure history | executor (#319) |
| `#319` | U2 | Submit executor honesty | **DONE** @ RFC-0204: env bind + explicit mock/reference label | AddressBook (#320) |
| `#320` | U2 | AddressBook selective rollback | **DONE** @ RFC-0205: selective candidate rollback; parallel upsert kept | docs (#321) |
| `#321` | U2 | systemd/docs prime-port | runbook + unit: bind проходить `validate_aira_port` | RFC close (#322) |
| `#322` | U | RFC-0198 + close | consolidating RFC-0198; QUEUE U closed; no OPEN U atoms | — |

```text
#313 U0 wiring (DONE this atom)
  → #314 U1 verify capsule-sourced DONE @ RFC-0199
    → #315 U1 result authority DONE @ RFC-0200
      → #316 U1 CLI identity DONE @ RFC-0201
        → #317 U1 policy audit IDs DONE @ RFC-0202
          → #318 U1 failed submit durable DONE @ RFC-0203
            → #319 U2 executor honesty DONE @ RFC-0204
              → #320 U2 AddressBook selective rollback DONE @ RFC-0205
                → #321 U2 systemd prime-port (first OPEN)
                  → #322 U close RFC-0198
```

**Anti-merge:** не зливати verify з result-index; не зливати identity з policy IDs; не зливати failed-submit persistence з policy uniqueness; не зливати executor honesty з AddressBook; не зливати docs prime-port з correctness atoms.

---

## 6. Acceptance Phase U (після активації й close)

```text
QUEUE `#313`–`#322` DONE; no OPEN U atoms.
Підміна output.expression+result (узгоджена, інша задача) → VerificationFailed; немає VRA.
Зміна лише ProblemRecord.result → get_result(problem_id) не повертає підміну як verified.
Повторний `aira identity create` на існуючій парі → error; secret/descriptor незмінні.
Два submit → обидві policy-події в durable log (різні ID); same-ID/different-hash → fail-closed.
Failed execution (напр. div/0) після reopen: problem + failure evidence присутні.
Штатний submit: або configured backend, або явний mock/reference режим у UI/CLI (без вигаданої готовності моделі).
Паралельно: dial fail rollback + інший upsert → другий peer лишається в book.
`deploy/systemd` + runbook peer listen: порт приймається prime-port validation.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no GUI/ledger deps.
Анти-місія не порушена; public bind не став default.
```

---

## 7. RFC (після активації)

- Consolidating (`#322`): **AIRA-RFC-0198** — id **confirmed free** 2026-09-10 (немає `AIRA-RFC-0198*` у дереві; file-free until close).
- Per-atom RFC-D: `#314`→**0199** … `#321`→**0206** (призначаються на кожному PR); `#322`→0198 consolidating.

---

## 8. Activation rule

```text
QUEUE T closed @ RFC-0192
  + developer «активувати Phase U» (2026-09-10)
  → wiring `#313` DONE; план IN PROGRESS; first OPEN `#314`.
```

---

## 9. Матриця evidence → атоми (з `aira-current.md` @ `ef5f69c`)

| Аудит § | Критерій | Атом |
|---------|----------|------|
| §2 P1 | verifier uses output expression | `#314` |
| §3 P1 | result-by-problem skips ArtifactStore | `#315` |
| §4 P1 | CLI identity create overwrites | `#316` |
| §5 P1 | policy audit ID collision / silent skip | `#317` |
| §6 P1 | failed submit not persisted | `#318` |
| §7 | mock-only submit / executor honesty | `#319` |
| §9 P2 | AddressBook full-snapshot rollback | `#320` |
| §11 docs | systemd `:7900` / prime-port | `#321` |
| §8 / §10 | O(N²) events, HTTP mutex, timeout, Timestamp/SQLite | **поза U** (наступний план) |
| §12–13 | non-goals / priority order | plan + close `#322` |
