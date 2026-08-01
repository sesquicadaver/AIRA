# AIRA — лінійна черга задач

**Оновлено:** 2026-08-01  
**Правило виконання:** завжди береться **перший OPEN** рядок; один рядок = один Analyze-цикл = одна атомарна зміна; не пропускати; не зливати два рядки в один PR; не чіпати `Manifesto etc/**`, `Meditation_About/**`.  
**Канон:** цей файл. Старі `Analyze-*/todo/TODO_FIXME.md` — лише provenance.

## Стан

| | |
|--|--|
| `main` | Analyze-58 CLOSED (`f1e311f`); QUEUE #23 DONE; next OPEN #24 |
| MVP / Peer P0–P2 micros #1–17 | **архів (DONE)** |
| Активна черга | **#24 → …** (перший OPEN = наступний цикл) |

## Правила атомарності

1. **Лінійність:** `#N` стартує лише після `#N-1` = DONE (APPROVE/CLEAR + UltraQA + push).
2. **Один вихід:** кожен рядок має один measurable «Done when».
3. **Не в scope:** колонка обов’язкова — усе інше відкладається в наступні рядки.
4. **Anti-merge:** Noise+NAT+DHT / dual-key+Noise / authn+federation — заборонені в одному рядку.
5. **Перенумерація:** нові задачі лише **в кінець** OPEN-хвоста; не вставляти між DONE і поточним OPEN без окремого рішення розробника.

---

## Архів (закрито) — Phase A: #1–17

Коротко: A-34…A-52 (listen → Noise → trust-delta → dual-key → notify → CSU publisher → audit → `.prev` → tenant keyring → gossip/relay/DHT → HTTP TLS/Bearer/mTLS → x25519 rotate → remote rekey grace → self-sovereign trust-delta).  
Деталі — у відповідних `Analyze-N/` і git history. **Не брати в роботу повторно.**

---

## Активна черга (лінійна) — Phase B

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 18 | **DONE** | ~~Analyze-53 — gossip drop non-self-sovereign~~ | Gossip: не форвардити `peer.trust.delta`, якщо `subject_id ≠ issuer` | тест + `docs/peer-link.md`; apply без змін політики A-52 | DHT book; relay persist |
| 19 | **DONE** | ~~Analyze-54 — x25519 peer notify~~ **WONT-NEED** | Hello already Ed25519-binds `x25519_pub_hex` each dial; no separate notify | docs + QUEUE rationale (no runtime notify) | pin cache; dual-static grace |
| 20 | **DONE** | ~~Analyze-55 — mTLS CN→TrustStore~~ | CN = full AiraRef; TrustStore exists + not revoked after CA | fail-closed tests + docs/local-node | optional client auth; окремий health |
| 21 | **DONE** | ~~Analyze-56 — separate health listener~~ | Окремий health listener **без** require client-cert (коли mTLS увімкнено на API) | `/health` reachable без клієнтського сертифіката; тести | CN map (вже #20); public bind |
| 22 | **DONE** | ~~Analyze-57 — DHT→address_book~~ | Opt-in: результат DHT find/announce → upsert у `address_book.json` | CLI flag + тест dial після upsert | discv5; auto без flag |
| 23 | **DONE** | ~~Analyze-58 — durable relay registry~~ | Durable relay hub registry на диску (пережив рестарт процесу) | reload після restart у тесті | STUN; session crypto change |
| 24 | OPEN | Analyze-59 | Concurrent per-connection recv tasks у `peer listen` | ≥2 паралельні сесії recv без блокування accept loop | systemd unit |
| 25 | OPEN | Analyze-60 | Приклад systemd/supervisor unit для `aira-node` / `peer listen` | файл(и) + короткий runbook у docs | код runtime |
| 26 | OPEN | Analyze-61 | Retention/prune для `.prev.<stamp>` слотів | CLI/policy + тести GC | per-CSU secrets |
| 27 | OPEN | Analyze-62 | Durable on-disk per-CSU signing secrets | load/save + ізоляція tenant | tenant rotate ceremony |
| 28 | OPEN | Analyze-63 | Tenant key rotate / revoke ceremony | CLI + audit + тести | HTTP authz |
| 29 | OPEN | Analyze-64 | Multi-tenant HTTP authz (Bearer/mTLS → tenant scope) | відмова cross-tenant; тести | federation |
| 30 | OPEN | Analyze-65 | YAML parity для `config.json` (читання `config.yaml`) | еквівалентний bootstrap; тест | SQLite audit rewrite |
| 31 | OPEN | Analyze-66 | STUN/ICE-lite для peer dial (один механізм, без full ICE stack) | documented dial path + тест/harness у venv | discv5 UDP |
| 32 | OPEN | Analyze-67 | UDP discv5-style announce (локальний) | announce+store; не iterative | FIND_NODE |
| 33 | OPEN | Analyze-68 | Iterative FIND_NODE поверх #32 | closest lookup over UDP path | federation join |
| 34 | OPEN | Analyze-69 | Public HTTP bind лише з **явним** opt-in + fail-closed default | docs + тест відмови без flag | federation |
| 35 | OPEN | Analyze-70 | Federation join prototype (roadmap v0.3 micro) | мінімальний join+trust; Living Spec | settlement/CRP |

### Наступний цикл

**#24 → Analyze-59** (concurrent per-connection recv).

---

## Джерела (мапінг Out → рядок)

| Джерело | Рядки |
|---------|-------|
| A-52 architect WATCH (gossip doomed deltas) | #18 |
| A-49 deferred x25519 notify | #19 **WONT-NEED** (Analyze-54: hello-sufficient) |
| A-51 / `docs/crypto.md` CN→TrustStore | #20 |
| A-51 separate health | #21 |
| `docs/peer-link.md` auto address-book from DHT | #22 |
| `docs/peer-link.md` durable relay store | #23 |
| A-34 concurrent recv / systemd | #24–25 |
| A-41 prune `.prev` | #26 |
| A-42 / crypto Out per-CSU secrets + ceremony | #27–28 |
| A-48 multi-tenant HTTP authz | #29 |
| `docs/local-node.md` YAML deferred | #30 |
| peer-link Out STUN / discv5 / FIND_NODE | #31–33 |
| peer-link public HTTP bind | #34 |
| `specs/mvp-roadmap.md` post-MVP federation | #35 |

Після DONE рядка: позначити `~~…~~ **DONE**`, оновити «Наступний цикл», закрити відповідний `Analyze-N/`.
