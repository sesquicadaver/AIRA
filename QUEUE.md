# AIRA — лінійна черга задач

**Оновлено:** 2026-08-19  
**Правило виконання:** завжди береться **перший OPEN** рядок; один рядок = один Analyze-цикл = одна атомарна зміна; не пропускати; не зливати два рядки в один PR; не чіпати `Manifesto etc/**`, `Meditation_About/**`.  
**Канон:** цей файл. Старі `analysis/Analyze-*/todo/TODO_FIXME.md` — лише provenance.  
**План етапу:** [`docs/phase-c-plan.md`](docs/phase-c-plan.md) — загальний план → атоми без перекриття → цей лінійний хвіст.

## Стан

| | |
|--|--|
| `main` | Analyze-79 CLOSED @ 611c1b5; QUEUE #44 DONE; наступний OPEN **#45** |
| MVP / Peer P0–P2 micros #1–17 | **архів (DONE)** |
| Phase B #18–#37 | **архів (DONE)** |
| Активна черга | Phase C **#45 OPEN** (Analyze-80) |

## Правила атомарності

1. **Лінійність:** `#N` стартує лише після `#N-1` = DONE (APPROVE/CLEAR + UltraQA + push).
2. **Один вихід:** кожен рядок має один measurable «Done when».
3. **Не в scope:** колонка обов’язкова — усе інше відкладається в наступні рядки.
4. **Anti-merge:** Noise+NAT+DHT / dual-key+Noise / authn+federation — заборонені в одному рядку. Phase C: не зливати типи дескрипторів, не зливати split різних `.rs`, не зливати CI з crypto.
5. **Перенумерація:** нові задачі лише **в кінець** OPEN-хвоста; не вставляти між DONE і поточним OPEN без окремого рішення розробника.
6. **План-перед-чергою:** поодинокі пункти в Phase C не додавати; хвіст `#38`–`#52` — повний етап. Після `#52` — новий загальний план, не новий поодинокий рядок.

---

## Архів (закрито) — Phase A: #1–17

Коротко: A-34…A-52 (listen → Noise → trust-delta → dual-key → notify → CSU publisher → audit → `.prev` → tenant keyring → gossip/relay/DHT → HTTP TLS/Bearer/mTLS → x25519 rotate → remote rekey grace → self-sovereign trust-delta).  
Деталі — у відповідних `analysis/Analyze-N/` і git history. **Не брати в роботу повторно.**

---

## Архів (закрито) — Phase B: #18–#37

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 18 | **DONE** | ~~Analyze-53 — gossip drop non-self-sovereign~~ | Gossip: не форвардити `peer.trust.delta`, якщо `subject_id ≠ issuer` | тест + `docs/peer-link.md`; apply без змін політики A-52 | DHT book; relay persist |
| 19 | **DONE** | ~~Analyze-54 — x25519 peer notify~~ **WONT-NEED** | Hello already Ed25519-binds `x25519_pub_hex` each dial; no separate notify | docs + QUEUE rationale (no runtime notify) | pin cache; dual-static grace |
| 20 | **DONE** | ~~Analyze-55 — mTLS CN→TrustStore~~ | CN = full AiraRef; TrustStore exists + not revoked after CA | fail-closed tests + docs/local-node | optional client auth; окремий health |
| 21 | **DONE** | ~~Analyze-56 — separate health listener~~ | Окремий health listener **без** require client-cert (коли mTLS увімкнено на API) | `/health` reachable без клієнтського сертифіката; тести | CN map (вже #20); public bind |
| 22 | **DONE** | ~~Analyze-57 — DHT→address_book~~ | Opt-in: результат DHT find/announce → upsert у `address_book.json` | CLI flag + тест dial після upsert | discv5; auto без flag |
| 23 | **DONE** | ~~Analyze-58 — durable relay registry~~ | Durable relay hub registry на диску (пережив рестарт процесу) | reload після restart у тесті | STUN; session crypto change |
| 24 | **DONE** | ~~Analyze-59 — concurrent accept (handshake off loop)~~ | TCP `accept_tcp` на циклі; spawn `complete_accept` (+recv/relay) | ≥2 паралельні сесії; hung handshake не блокує accept; тести | systemd unit |
| 25 | **DONE** | ~~Analyze-60 — systemd examples~~ | Приклад systemd units для `aira-node` / `peer listen` + runbook | `deploy/systemd/*.service` + `docs/runbook-systemd.md` | код runtime |
| 26 | **DONE** | ~~Analyze-61 — retention/prune `.prev.<stamp>`~~ | CLI/policy GC archived stamp slots (ed25519+x25519); never latest | `identity backups prune` + тести | per-CSU secrets |
| 27 | **DONE** | ~~Analyze-62 — durable per-CSU secrets~~ | On-disk `identity/tenants/<hex>/` load/save + isolation + auto-load | тести + CLI `identity csu-tenant` | tenant rotate ceremony |
| 28 | **DONE** | ~~Analyze-63 — tenant rotate/revoke ceremony~~ | Same-publisher rotate + revoke unload/delete + audit | CLI + audit + тести | HTTP authz |
| 29 | **DONE** | ~~Analyze-64 — multi-tenant HTTP authz~~ | Bearer map → publisher; CSU register/list cross-tenant 403 | тести + docs/local-node | federation; mTLS CN seam |
| 30 | **DONE** | ~~Analyze-65 — YAML config read parity~~ | `config.yaml` xor `config.json` → `NodeConfig`; init JSON-only | тести + docs/local-node | SQLite audit rewrite |
| 31 | **DONE** | ~~Analyze-66 — STUN Binding reflexive~~ | Binding → `stun_reflexive.json` → `dht announce --from-stun`; dial unchanged | mock STUN + docs/peer-link | discv5 UDP |
| 32 | **DONE** | ~~Analyze-67 — UDP discv5-style announce~~ | Signed UDP announce → local `dht.json` (`source=udp`); no FIND_NODE | тести + docs/peer-link | FIND_NODE |
| 33 | **DONE** | ~~Analyze-68 — iterative FIND_NODE~~ | Signed UDP FIND/NODES; XOR iterate; merge trusted hints | тести + docs/peer-link | federation join |
| 34 | **DONE** | ~~Analyze-69 — public HTTP bind opt-in~~ | Public HTTP bind лише з **явним** opt-in + fail-closed default | docs + тест відмови без flag | Federation |
| 35 | **DONE** | ~~Analyze-70 — federation join prototype~~ | Federation join prototype (roadmap v0.3 micro) | мінімальний join+trust; Living Spec | settlement/CRP |
| 36 | **DONE** | ~~Analyze-71 — tenant `.prev` prune~~ | Tenant `ed25519.prev.<stamp>` prune | prune CLI parity з node backups; тести | HTTP authz; stdin secret |
| 37 | **DONE** | ~~Analyze-72 — tenant `--secret-hex-file` / stdin~~ | Tenant `--secret-hex-file` / stdin | register/rotate читає secret з file або stdin, не argv | HTTP authz |

---

## Активна черга (лінійна) — Phase C: стабілізація Reference

План: [`docs/phase-c-plan.md`](docs/phase-c-plan.md). Не зливати рядки. Не додавати фічі поза `#38`–`#52`.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 38 | **DONE** | ~~Analyze-73 — CI schema/C0/C1 gate~~ | GitHub Actions: обов’язкові `schema validate --fixtures` + conformance C0 + C1 | CI червоний при невалідній схемі або провалі C0/C1; зелений на поточному дереві | семантика схем; C2 у CI; підписи; split файлів |
| 39 | **DONE** | ~~Analyze-74 — canonical descriptor hash helper~~ | Примітив hash/sign/verify canonical descriptor **без** зміни production call-sites | тести helper; verify-шляхи runtime не змінені | перемикання Event/Artifact/Object/CSU; прибирання fallback |
| 40 | **DONE** | ~~Analyze-75 — Event canonical signatures~~ | Event sign/verify = canonical descriptor без `signature`; mutation-тести; немає event-path runtime fallback на `LOCAL_TEST_DOMAIN_MSG` | зміна `event_type` / `causal_refs` / `object_refs` / `artifact_refs` / `payload_hash` ламає verify | Artifact/Object/CSU; protocol envelope як окремий sweep; Noise |
| 41 | **DONE** | ~~Analyze-76 — Artifact canonical signatures~~ | Artifact descriptor: canonical hash-signing + mutation-тести | зміна артефактних полів дескриптора ламає verify | Event/Object/CSU; CAS layout |
| 42 | **DONE** | ~~Analyze-77 — Object canonical signatures~~ | Core Object descriptor: canonical hash-signing + mutation-тести | зміна об’єктних полів дескриптора ламає verify | Event/Artifact/CSU |
| 43 | **DONE** | ~~Analyze-78 — CSU manifest canonical signatures~~ | Manifest sign/verify = canonical manifest без `signature` + mutation-тести | зміна полів маніфесту ламає verify | Event/Artifact/Object; нові CSU |
| 44 | **DONE** | ~~Analyze-79 — leftover test-sig fallback~~ | Прибрати runtime fallback `LOCAL_TEST_DOMAIN_MSG`/TESTSIG поза `#[cfg(test)]` там, де лишилось після #40–#43 | runtime verify не приймає test-domain fallback | повторний перепис уже мігрованих дескрипторів; нові протоколи |
| 45 | OPEN | Analyze-80 — core dependency firewall | CI: `aira-core` ↛ node/peer/concrete CSU; CSU ↛ CSU; без циклів імпорту | CI падає на forbidden import / цикл | split файлів; нові crates |
| 46 | OPEN | Analyze-81 — modularize aira-cli | Mechanical split `crates/aira-cli/src/main.rs` → `commands/*` | CLI поведінка незмінна; тести CLI зелені | crypto/tenant/http/tls; нові підкоманди |
| 47 | OPEN | Analyze-82 — modularize crypto.rs | Mechanical split `crates/aira-object/src/crypto.rs` | тести object/crypto зелені | `tenant.rs`; CLI; HTTP |
| 48 | OPEN | Analyze-83 — modularize tenant.rs | Mechanical split `crates/aira-object/src/tenant.rs` | тести tenant зелені | `crypto.rs`; зміна HTTP authz семантики |
| 49 | OPEN | Analyze-84 — modularize http.rs | Mechanical split `crates/aira-node/src/http.rs` | HTTP тести/поведінка незмінні | `tls.rs`; нові маршрути |
| 50 | OPEN | Analyze-85 — modularize tls.rs | Mechanical split `crates/aira-node/src/tls.rs` | TLS/mTLS/health bind незмінні | `http.rs`; нові режими TLS |
| 51 | OPEN | Analyze-86 — OperationalPlane reference-local | Docs + коментарі модуля: plane = C1 reference/demo, не production runtime | явно зафіксовано non-production статус | зміна drain/loop семантики |
| 52 | OPEN | Analyze-87 — implementation status matrix | `docs/implementation-status.md`: ТЗ → модуль → тести → статус | покриття Book 0–IV, schemas, C0/C1, CSU basic, HTTP/peer як post-MVP | нова реалізація «щоб заповнити матрицю» |

### Наступний цикл

**#45 OPEN** — Analyze-80: core dependency firewall.

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
| A-63 deferred tenant `.prev` prune | #36 |
| A-62/A-63 deferred tenant stdin/`--secret-hex-file` | #37 **DONE** (Analyze-72) |
| A-48 multi-tenant HTTP authz | #29 |
| `docs/local-node.md` YAML deferred | #30 |
| peer-link Out STUN / discv5 / FIND_NODE | #31–33 |
| peer-link public HTTP bind | #34 **DONE** (Analyze-69) |
| `specs/mvp-roadmap.md` post-MVP federation | #35 **DONE** (Analyze-70) |
| EVO-1/EVO-2 CI schema+C0+C1 | #38 |
| EVO-1/EVO-2 canonical signatures (розщеплено по типах) | #39–#44 |
| EVO-2 dependency firewall | #45 |
| EVO-1/EVO-2 modularize monoliths (розщеплено по файлах) | #46–#50 |
| EVO-2 OperationalPlane reference-local | #51 |
| EVO-2 implementation-status matrix | #52 |

Після DONE рядка: позначити `~~…~~ **DONE**`, оновити «Наступний цикл», закрити відповідний `analysis/Analyze-N/`.
