# AIRA — лінійна черга задач

**Оновлено:** 2026-08-20  
**Правило виконання:** завжди береться **перший OPEN** рядок; один рядок = один Analyze-цикл = одна атомарна зміна; не пропускати; не зливати два рядки в один PR; не чіпати `Manifesto etc/**`, `Meditation_About/**`.  
**Канон:** цей файл. Старі `analysis/Analyze-*/todo/TODO_FIXME.md` — лише provenance.  
**План етапу:** Phase C [`docs/phase-c-plan.md`](docs/phase-c-plan.md) **DONE**. Phase D [`docs/phase-d-plan.md`](docs/phase-d-plan.md) — D0–D6 (`#53`–`#71`) **DONE**. Addendum D7 `#72`–`#73` **DONE**; OPEN `#74`. D4/D5 уже CLOSED.

## Стан

| | |
|--|--|
| `main` | Analyze-108 CLOSED @ 9760921; QUEUE #73 DONE; Addendum D7 **#74 OPEN** |
| MVP / Peer P0–P2 micros #1–17 | **архів (DONE)** |
| Phase B #18–#37 | **архів (DONE)** |
| Активна черга | Phase D Addendum D7 **#74 OPEN** (Analyze-109 — CLI models recommend) |

## Правила атомарності

1. **Лінійність:** `#N` стартує лише після `#N-1` = DONE (APPROVE/CLEAR + UltraQA + push).
2. **Один вихід:** кожен рядок має один measurable «Done when».
3. **Не в scope:** колонка обов’язкова — усе інше відкладається в наступні рядки.
4. **Anti-merge:** Noise+NAT+DHT / dual-key+Noise / authn+federation — заборонені в одному рядку. Phase C: не зливати типи дескрипторів, не зливати split різних `.rs`, не зливати CI з crypto. Phase D: не зливати різні payload schema files; не зливати schema з downloader.
5. **Перенумерація:** нові задачі лише **в кінець** OPEN-хвоста; не вставляти між DONE і поточним OPEN без окремого рішення розробника.
6. **План-перед-чергою:** поодинокі пункти не додавати. Phase C хвіст `#38`–`#52` — повний етап. Phase D — [`docs/phase-d-plan.md`](docs/phase-d-plan.md); `#53`–`#71` DONE; Addendum D7 `#72`–`#74` відкрито 2026-08-20.

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

## Архів (закрито) — Phase C: стабілізація Reference

План: [`docs/phase-c-plan.md`](docs/phase-c-plan.md). **DONE** `#38`–`#52`.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 38 | **DONE** | ~~Analyze-73 — CI schema/C0/C1 gate~~ | GitHub Actions: обов’язкові `schema validate --fixtures` + conformance C0 + C1 | CI червоний при невалідній схемі або провалі C0/C1; зелений на поточному дереві | семантика схем; C2 у CI; підписи; split файлів |
| 39 | **DONE** | ~~Analyze-74 — canonical descriptor hash helper~~ | Примітив hash/sign/verify canonical descriptor **без** зміни production call-sites | тести helper; verify-шляхи runtime не змінені | перемикання Event/Artifact/Object/CSU; прибирання fallback |
| 40 | **DONE** | ~~Analyze-75 — Event canonical signatures~~ | Event sign/verify = canonical descriptor без `signature`; mutation-тести; немає event-path runtime fallback на `LOCAL_TEST_DOMAIN_MSG` | зміна `event_type` / `causal_refs` / `object_refs` / `artifact_refs` / `payload_hash` ламає verify | Artifact/Object/CSU; protocol envelope як окремий sweep; Noise |
| 41 | **DONE** | ~~Analyze-76 — Artifact canonical signatures~~ | Artifact descriptor: canonical hash-signing + mutation-тести | зміна артефактних полів дескриптора ламає verify | Event/Object/CSU; CAS layout |
| 42 | **DONE** | ~~Analyze-77 — Object canonical signatures~~ | Core Object descriptor: canonical hash-signing + mutation-тести | зміна об’єктних полів дескриптора ламає verify | Event/Artifact/CSU |
| 43 | **DONE** | ~~Analyze-78 — CSU manifest canonical signatures~~ | Manifest sign/verify = canonical manifest без `signature` + mutation-тести | зміна полів маніфесту ламає verify | Event/Artifact/Object; нові CSU |
| 44 | **DONE** | ~~Analyze-79 — leftover test-sig fallback~~ | Прибрати runtime fallback `LOCAL_TEST_DOMAIN_MSG`/TESTSIG поза `#[cfg(test)]` там, де лишилось після #40–#43 | runtime verify не приймає test-domain fallback | повторний перепис уже мігрованих дескрипторів; нові протоколи |
| 45 | **DONE** | ~~Analyze-80 — core dependency firewall~~ | CI: `aira-core` ↛ node/peer/concrete CSU; CSU ↛ CSU; без циклів імпорту | CI падає на forbidden import / цикл | split файлів; нові crates |
| 46 | **DONE** | ~~Analyze-81 — modularize aira-cli~~ | Mechanical split `crates/aira-cli/src/main.rs` → `commands/*` | CLI поведінка незмінна; тести CLI зелені | crypto/tenant/http/tls; нові підкоманди |
| 47 | **DONE** | ~~Analyze-82 — modularize crypto.rs~~ | Mechanical split `crates/aira-object/src/crypto.rs` | тести object/crypto зелені | `tenant.rs`; CLI; HTTP |
| 48 | **DONE** | ~~Analyze-83 — modularize tenant.rs~~ | Mechanical split `crates/aira-object/src/tenant.rs` | тести tenant зелені | `crypto.rs`; зміна HTTP authz семантики |
| 49 | **DONE** | ~~Analyze-84 — modularize http.rs~~ | Mechanical split `crates/aira-node/src/http.rs` | HTTP тести/поведінка незмінні | `tls.rs`; нові маршрути |
| 50 | **DONE** | ~~Analyze-85 — modularize tls.rs~~ | Mechanical split `crates/aira-node/src/tls.rs` | TLS/mTLS/health bind незмінні | `http.rs`; нові режими TLS |
| 51 | **DONE** | ~~Analyze-86 — OperationalPlane reference-local~~ | Docs + коментарі модуля: plane = C1 reference/demo, не production runtime | явно зафіксовано non-production статус | зміна drain/loop семантики |
| 52 | **DONE** | ~~Analyze-87 — implementation status matrix~~ | `docs/implementation-status.md`: ТЗ → модуль → тести → статус | покриття Book 0–IV, schemas, C0/C1, CSU basic, HTTP/peer як post-MVP | нова реалізація «щоб заповнити матрицю» |

---

## Активна черга (лінійна) — Phase D: Model Artifact & Inventory

План: [`docs/phase-d-plan.md`](docs/phase-d-plan.md). Не зливати рядки. Envelope = `CustomArtifact` + payload `$id`; не розширювати canonical `ArtifactType` без окремого RFC-S. Addendum D5: share opt-in only; no global registry / remote advertise in this wave.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 53 | **DONE** | ~~Analyze-88 — ModelArtifact payload schema~~ | RFC-S + `aira:schema:model:artifact:0.1` + fixtures | schema validate --fixtures зелений; enum `ArtifactType` незмінний | Profile/Inventory/Policy schemas; downloader |
| 54 | **DONE** | ~~Analyze-89 — ModelProfile payload schema~~ | `aira:schema:model:profile:0.1` + fixtures | schema + fixtures | Inventory; hardware scan |
| 55 | **DONE** | ~~Analyze-90 — LocalModelInventory payload schema~~ | `aira:schema:model:inventory:0.1` + fixtures | schema + fixtures | CLI scan; мережа |
| 56 | **DONE** | ~~Analyze-91 — ModelCompatibilityEvidence schema~~ | payload evidence: reason / confidence / scope | schema + fixtures | auto-download; rating score |
| 57 | **DONE** | ~~Analyze-92 — ModelAcquisitionPolicy schema~~ | payload policy; default `auto_download=false` | schema + fixtures | downloader; allowlist runtime |
| 58 | **DONE** | ~~Analyze-93 — local inventory read-only~~ | `aira models scan\|list`; Inventory CSU, scoped FS | scan без мережі/download; immutable inventory artifact | compatible resolver; download |
| 59 | **DONE** | ~~Analyze-94 — compatibility resolver~~ | hardware/backend/profile → runnable / incompatible / unknown | Evidence Artifact + reason; без download | acquisition CSU; CLI download |
| 60 | **DONE** | ~~Analyze-95 — acquisition policy DENY~~ | runtime default-deny download | без policy = DENY + Event/Evidence; implicit download немає | реальний download (D4); sharing |
| 61 | **DONE** | ~~Analyze-96 — policy ALLOW path~~ | `auto_download=true` → decision ALLOW (+ Event); DENY `#60` збережено; без byte transfer | quarantine fetch; verify; activate; HTTP |
| 62 | **DONE** | ~~Analyze-97 — quarantine fetch (local source)~~ | після ALLOW: `--source` → `<root>/models/quarantine/`; Event; без activation | hash verify; activate; remote URL |
| 63 | **DONE** | ~~Analyze-98 — verify hash/signature~~ | mismatch → reject + Evidence; match → `verified/` staging | activate; inventory promote; HTTP |
| 64 | **DONE** | ~~Analyze-99 — activate verified model~~ | explicit activate → cache + inventory + Event; без auto-execution | sharing (D5); rating (D6); remote registry |
| 65 | **DONE** | ~~Analyze-100 — ModelShareOffer schema~~ | `aira:schema:model:share-offer:0.1` + fixtures; CustomArtifact envelope | policy runtime; publish CLI; network |
| 66 | **DONE** | ~~Analyze-101 — share_custom_models gate~~ | absent/false → DENY publish + Event; true → ALLOW (без publish) | local publish; capability ad; rating |
| 67 | **DONE** | ~~Analyze-102 — local publish signed descriptor~~ | cache → signed ModelArtifact + ShareOffer; Event; без remote push | capability ad; remote registry; rating |
| 68 | **DONE** | ~~Analyze-103 — local capability advertisement~~ | capability CustomArtifact + CLI publish\|share; local-only | federation; DHT; rating (D6); marketplace |
| 69 | **DONE** | ~~Analyze-104 — ModelRatingEvidence schema~~ | `aira:schema:model:rating-evidence:0.1` + fixtures; context-bound; не global score | rating CSU; CLI rate; recommendation (D7) |
| 70 | **DONE** | ~~Analyze-105 — local rating evidence publish~~ | signed rating CustomArtifact + Event; context required | CLI; recommendation; network scoreboard |
| 71 | **DONE** | ~~Analyze-106 — CLI `aira models rate`~~ | publish contextual rating evidence; local-only | D7 recommend; marketplace; global rank |
| 72 | **DONE** | ~~Analyze-107 — UpgradeRecommendation schema~~ | `aira:schema:model:upgrade-recommendation:0.1` + fixtures; advisory | recommend CSU; CLI; marketplace |
| 73 | **DONE** | ~~Analyze-108 — local upgrade recommendation publish~~ | signed recommendation CustomArtifact + Event; evidence-backed | CLI; settlement; ads |
| 74 | OPEN | Analyze-109 — CLI `aira models recommend` | advisory recommendation; local-only | marketplace; auto-download; settlement |

### Наступний цикл

**#74 OPEN** — Analyze-109: CLI `aira models recommend` (D7.3).

> Порядок: D6 DONE → D7; D4/D5 уже DONE (не переробляти).

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
| EVO-3 model layer (CSU/Artifact, не Core) | план [`docs/phase-d-plan.md`](docs/phase-d-plan.md); `#53`–`#72` DONE; Addendum D7 OPEN `#73`–`#74` |

Після DONE рядка: позначити `~~…~~ **DONE**`, оновити «Наступний цикл», закрити відповідний `analysis/Analyze-N/`.
