# AIRA — лінійна черга задач

**Оновлено:** 2026-09-05  
**Правило виконання:** завжди береться **перший OPEN** рядок; один рядок = один Analyze-цикл = одна атомарна зміна; не пропускати; не зливати два рядки в один PR; не чіпати `Manifesto etc/**`, `Meditation_About/**`.  
**Канон:** цей файл. Старі `analysis/Analyze-*/todo/TODO_FIXME.md` — лише provenance.  
**План етапу:** Phase C [`docs/phase-c-plan.md`](docs/phase-c-plan.md) **DONE**. Phase D [`docs/phase-d-plan.md`](docs/phase-d-plan.md) — D0–D7 (`#53`–`#74`) **DONE**. Phase E [`docs/phase-e-plan.md`](docs/phase-e-plan.md) + UX [`docs/desktop-ux.md`](docs/desktop-ux.md) — `#75`–`#106` **DONE** (2026-08-22). Phase F Post-E stabilization [`docs/phase-f-plan.md`](docs/phase-f-plan.md) — `#107`–`#119` **DONE** (2026-08-24). Phase G Reference v0.2 [`docs/phase-g-plan.md`](docs/phase-g-plan.md) — `#120`–`#151` **DONE** (2026-08-28, RFC-0069). Phase H Protocol depth v0.3 [`docs/phase-h-plan.md`](docs/phase-h-plan.md) — `#152`–`#183` **DONE** (2026-08-30, RFC-0077). Phase I Semantic contract stabilization [`docs/phase-i-plan.md`](docs/phase-i-plan.md) — `#184`–`#198` **DONE** (2026-08-30, RFC-0078). Phase J Book-gap local remainder [`docs/phase-j-plan.md`](docs/phase-j-plan.md) — `#199`–`#208` **DONE** @ RFC-0096. QUEUE J closed. Phase K Local LLM Execution CSU [`docs/phase-k-plan.md`](docs/phase-k-plan.md) — `#209`–`#216` **DONE** @ RFC-0104. QUEUE K closed. Phase L Execution-boundary hardening [`docs/phase-l-plan.md`](docs/phase-l-plan.md) — `#217`–`#223` **DONE** @ RFC-0111. QUEUE L closed. Phase M OS child sandbox [`docs/phase-m-plan.md`](docs/phase-m-plan.md) — `#224`–`#230` **DONE** @ RFC-0117. QUEUE M closed. Phase N Global Node Rendezvous [`docs/phase-n-plan.md`](docs/phase-n-plan.md) — `#231`–`#233` **DONE**; `#234`–`#247` OPEN (RFC-0123 reserved; RFC-0124/0125).

## Стан

| | |
|--|--|
| `main` | Phase N `#231`–`#233` **DONE**; first OPEN `#234`; M `#224`–`#230` **DONE** @ RFC-0117; QUEUE M closed |
| MVP / Peer P0–P2 micros #1–17 | **архів (DONE)** |
| Phase B #18–#37 | **архів (DONE)** |
| Активна черга | Phase N `#234` (перший OPEN) |

## Правила атомарності

1. **Лінійність:** `#N` стартує лише після `#N-1` = DONE (APPROVE/CLEAR + UltraQA + push).
2. **Один вихід:** кожен рядок має один measurable «Done when».
3. **Не в scope:** колонка обов’язкова — усе інше відкладається в наступні рядки.
4. **Anti-merge:** Noise+NAT+DHT / dual-key+Noise / authn+federation — заборонені в одному рядку. Phase C: не зливати типи дескрипторів, не зливати split різних `.rs`, не зливати CI з crypto. Phase D: не зливати різні payload schema files; не зливати schema з downloader.
5. **Перенумерація:** нові задачі лише **в кінець** OPEN-хвоста; не вставляти між DONE і поточним OPEN без окремого рішення розробника.
6. **План-перед-чергою:** поодинокі пункти не додавати. Phase C `#38`–`#52` DONE. Phase D `#53`–`#74` DONE. Phase E `#75`–`#106` DONE. Phase F `#107`–`#119` DONE. Phase G — [`docs/phase-g-plan.md`](docs/phase-g-plan.md); `#120`–`#151` **DONE** (2026-08-28). Phase H — [`docs/phase-h-plan.md`](docs/phase-h-plan.md); `#152`–`#183` **DONE** (RFC-0077). Phase I — [`docs/phase-i-plan.md`](docs/phase-i-plan.md); `#184`–`#198` **DONE** (RFC-0078). Phase J — [`docs/phase-j-plan.md`](docs/phase-j-plan.md); `#199`–`#208` **DONE** (RFC-0096). Phase K — [`docs/phase-k-plan.md`](docs/phase-k-plan.md); `#209`–`#216` **DONE** (RFC-0104). Phase L — [`docs/phase-l-plan.md`](docs/phase-l-plan.md); `#217`–`#223` **DONE** @ RFC-0111. QUEUE L closed. Phase M — [`docs/phase-m-plan.md`](docs/phase-m-plan.md); `#224`–`#230` **DONE** @ RFC-0117. QUEUE M closed. Phase N — [`docs/phase-n-plan.md`](docs/phase-n-plan.md); `#231`–`#233` **DONE**; `#234` OPEN (RFC-0123; RFC-0124/0125).

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

## Архів (закрито) — Phase D: Model Artifact & Inventory

План: [`docs/phase-d-plan.md`](docs/phase-d-plan.md). **DONE** `#53`–`#74` (D0–D7). D4/D5 були CLOSED раніше (не перероблялись). Envelope = `CustomArtifact` + payload `$id`; не розширювати canonical `ArtifactType` без окремого RFC-S.

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
| 74 | **DONE** | ~~Analyze-109 — CLI `aira models recommend`~~ | advisory recommendation; local-only | marketplace; auto-download; settlement |

### Наступний цикл (Phase D)

Phase D **COMPLETE** (`#53`–`#74`). Далі — Phase E нижче.

---

## Активна черга (лінійна) — Phase E: Desktop UX (Linux P0)

План: [`docs/phase-e-plan.md`](docs/phase-e-plan.md) (lifecycle/port/HTTP/auth/layout §2). Рішення: [`docs/desktop-ux.md`](docs/desktop-ux.md). Provenance: [`NEXT_PROBLEM.md`](NEXT_PROBLEM.md) **RESOLVED**.  
E1 = **лише P0** / Developer Preview; P1+QR = Addendum E1.1 після `#79`. macOS/Windows = E2/E3 після Linux. **Не** вставляти stabilization phase перед `#75` — вимоги fold-in у Done when нижче.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 75 | **DONE** | ~~Analyze-110 — Desktop settings schema~~ | `aira:schema:desktop:settings:0.1` + fixtures (P0 fields + `instance_id` + auth placeholders; phase-e §2.1/§2.4) | schema validate --fixtures зелений | orchestrator; GUI; token generation; P1+ |
| 76 | **DONE** | ~~Analyze-111 — shared lifecycle + `aira desktop`~~ | `start\|stop\|status`; P0 layout; init+identity+loopback HTTP; §2.2–§2.4 (idempotent/lock/health/port attach\|fail; Desktop auth contract); shared lib | CLI + lifecycle tests | tray; AppImage; peer; OS autostart hooks |
| 77 | **DONE** | ~~Analyze-112 — Linux `.desktop` launcher~~ | `.desktop` → desktop start (shared lib/CLI); stop via CLI/tray docs | клік/меню запускає P0 node | tray UI; packaging; macOS/Win |
| 78 | **DONE** | ~~Analyze-113 — Linux tray/GUI (Rust-only)~~ | Status + native Open UI + Settings + Quit; shared lib; **XDG autostart hooks** (§2.5) | GUI start/stop; settings persist; autostart on/off works | P1; AppImage; Tauri/web; інші ОС |
| 79 | **DONE** | ~~Analyze-114 — Linux packaging~~ | tarball+`.desktop` + Developer Preview docs; Dev CLI notes; paths for `#78` | install без `cargo`; autostart paths valid | macOS DMG; Win installer; P1; AppImage |

## Активна черга (лінійна) — Phase E Addendum E1.1: P1 + файл/QR

План: [`docs/phase-e-plan.md`](docs/phase-e-plan.md) §4a. UX: [`docs/desktop-ux.md`](docs/desktop-ux.md) §3 (P1).  
**Не** P2–P6 / DHT / relay / gossip / camera / E2–E3 у цьому хвості.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 80 | **DONE** | ~~Analyze-115 — PeerInvite schema~~ | `aira:schema:desktop:peer-invite:0.1` + fixtures (identity_ref/pubkey + optional addr) | schema validate --fixtures зелений | settings P1; peer process; QR; GUI |
| 81 | **DONE** | ~~Analyze-116 — Settings P1 + peer_listen~~ | runtime приймає P1; default/validate `peer_listen`; P2+ fail-closed; docs | settings load/save P1 + тести | peer supervise; invite IO; GUI |
| 82 | **DONE** | ~~Analyze-117 — peer lifecycle supervise~~ | при P1: start/stop/status `peer listen --recv`; PID/lock; HTTP лишається | lifecycle тести dual-process | invite IO; QR; GUI |
| 83 | **DONE** | ~~Analyze-118 — invite file export/import~~ | export JSON; import → trust add + book upsert; CLI/shared lib | roundtrip тести | QR; GUI |
| 84 | **DONE** | ~~Analyze-119 — invite QR PNG~~ | PeerInvite ↔ PNG encode/decode (файл) | roundtrip тести | camera; GUI |
| 85 | **DONE** | ~~Analyze-120 — GUI P1 + invite UX~~ | profile toggle; peer status; export/import/QR file | GUI smoke P1 onboarding | P2+; camera; інші ОС |

### Наступний цикл

**E1.1 DONE** (`#80`–`#85`). Див. Addendum E2 нижче.

> Порядок E1.1: `#80` → `#81` → `#82` → `#83` → `#84` → `#85`.

## Активна черга (лінійна) — Phase E Addendum E2: macOS

План: [`docs/phase-e-plan.md`](docs/phase-e-plan.md) §4b.  
**Не** DMG/notarize/App Store / E3 у цьому хвості. Packaging = `.app` + tarball.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 86 | **DONE** | ~~Analyze-121 — macOS DesktopPaths~~ | Application Support / Preferences / Logs layout | unit тести layout | LaunchAgent; `.app`; DMG |
| 87 | **DONE** | ~~Analyze-122 — macOS LaunchAgent~~ | plist autostart за `autostart_on_login`; Linux XDG лишається | roundtrip тести plist | `.app` bundle; notarize |
| 88 | **DONE** | ~~Analyze-123 — macOS `.app` tarball~~ | package script + Info.plist + Contents/MacOS | tarball містить runnable layout | DMG; notarize; Windows |
| 89 | **DONE** | ~~Analyze-124 — macOS docs~~ | packaging/install docs + RFC | docs узгоджені з `#86`–`#88` | E3; App Store |

### Наступний цикл (E2)

**E2 macOS DONE** (`#86`–`#89`). Див. Addendum E3 нижче.

## Активна черга (лінійна) — Phase E Addendum E3: Windows

План: [`docs/phase-e-plan.md`](docs/phase-e-plan.md) §4c.  
**Не** MSI/codesign/Store у цьому хвості. Packaging = `.zip` + install script.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 90 | **DONE** | ~~Analyze-125 — Windows DesktopPaths~~ | `%LOCALAPPDATA%` / `%APPDATA%` layout | unit тести layout | autostart; zip; MSI |
| 91 | **DONE** | ~~Analyze-126 — Windows login autostart~~ | Startup/Registry за `autostart_on_login`; Linux/macOS лишаються | roundtrip тести hook | zip package; codesign |
| 92 | **DONE** | ~~Analyze-127 — Windows zip package~~ | package script + `deploy/windows/` install | zip містить runnable layout | MSI; codesign; macOS |
| 93 | **DONE** | ~~Analyze-128 — Windows docs~~ | packaging/install docs + RFC | docs узгоджені з `#90`–`#92` | App Store; MSI |

### Наступний цикл (E3)

**E3 Windows DONE** (`#90`–`#93`). Див. Addendum E4 нижче.

## Активна черга (лінійна) — Phase E Addendum E4: network profiles P2–P6

План: [`docs/phase-e-plan.md`](docs/phase-e-plan.md) §4d. UX: [`docs/desktop-ux.md`](docs/desktop-ux.md) §3.  
**Не** змішувати P3 relay + P4 gossip в одному рядку; **не** публічний STUN default; **не** auto-trust; **не** новий peer protocol.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 94 | **DONE** | ~~Analyze-129 — Settings P2~~ | runtime приймає P2; `peer_listen` як P1; P3+ fail-closed; docs | settings load/save P2 + тести | peer `--dht`; GUI |
| 95 | **DONE** | ~~Analyze-130 — Lifecycle P2 DHT~~ | supervise `peer listen --recv --dht --apply-book`; PID/lock | dual-root DHT→book smoke | P3 relay; GUI |
| 96 | **DONE** | ~~Analyze-131 — GUI P2~~ | profile selector P0\|P1\|P2; peer status (dht) | GUI smoke P2 | P3+; camera |
| 97 | **DONE** | ~~Analyze-132 — Settings P3 relay TTL~~ | runtime P3; `relay_ttl_days` default 31; P4+ fail-closed; P3\|P4 mutex у normalize | settings + mutex тести | peer `--relay`; GUI |
| 98 | **DONE** | ~~Analyze-133 — Lifecycle P3 relay~~ | supervise `--relay --relay-ttl-days N` | relay register/deliver smoke | gossip; GUI |
| 99 | **DONE** | ~~Analyze-134 — GUI P3 Advanced~~ | Advanced relay toggle/status; mutex hint P3 vs P4 | GUI smoke P3 | P4 gossip; P5 |
| 100 | **DONE** | ~~Analyze-135 — Settings P4 gossip~~ | runtime P4; P3\|P4 mutex; P5+ fail-closed | settings + mutex тести | peer `--gossip`; GUI |
| 101 | **DONE** | ~~Analyze-136 — Lifecycle P4 gossip~~ | supervise `--gossip` (без `--relay`) | gossip forward-filter smoke | relay; GUI |
| 102 | **DONE** | ~~Analyze-137 — GUI P4 Advanced~~ | gossip toggle/status у Advanced | GUI smoke P4 | P5; P6 |
| 103 | **DONE** | ~~Analyze-138 — Federation join desktop~~ | shared lib: descriptor file → `federation join`; membership read API | join pin + trust smoke | GUI wizard; Book II |
| 104 | **DONE** | ~~Analyze-139 — GUI P5 federation~~ | wizard import descriptor; membership status | GUI smoke P5 | P6; remote federation |
| 105 | **DONE** | ~~Analyze-140 — P6 Advanced discovery~~ | opt-in Dev panel: STUN/discv/FIND shortcuts; no public STUN default | operator smoke + docs hook | auto-trust; camera |
| 106 | **DONE** | ~~Analyze-141 — E4 docs + RFC~~ | `docs/desktop-network-profiles.md` + RFC-0043 | docs узгоджені з `#94`–`#105` | peer protocol change |

### Наступний цикл (E4)

**E4 завершено** (`#94`–`#106` DONE). Див. Phase F нижче.

> Mutex: P3 (relay) і P4 (gossip) — окремі lifecycle рядки; settings normalize fail-closed при одночасному виборі.

## Активна черга (лінійна) — Phase F: Post-E Stabilization

План: [`docs/phase-f-plan.md`](docs/phase-f-plan.md). Джерела: [`phase-e-plan.md`](docs/phase-e-plan.md) §«Пізніше»; [`implementation-status.md`](docs/implementation-status.md); EVO-2 stabilize→verify→harden C2.  
**Не** нові peer/federation фічі; **не** CRP/settlement/PHM; mechanical split без зміни поведінки.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 107 | **DONE** | ~~Analyze-142 — toolchain pin~~ | pin stable у `rust-toolchain.toml`; CI/docs parity | локально + CI використовують pinned channel | нові фічі; clippy rule change |
| 108 | **DONE** | ~~Analyze-143 — schema manifest gaps~~ | `identifier`, `epistemic-assessment`, `context-artifact` fixtures + manifest | `schema validate --fixtures` зелений | семантика схем; C2 |
| 109 | **DONE** | ~~Analyze-144 — CI governance doc~~ | `docs/ci-governance.md`: required checks, merge policy для `main` | doc узгоджений з `.github/workflows/ci.yml` | GitHub UI-only settings без doc |
| 110 | **DONE** | ~~Analyze-145 — SECURITY.md refresh~~ | актуальна security posture (CSU isolation, Desktop, reporting) | не «skeleton only»; cross-links docs | нові security features |
| 111 | **DONE** | ~~Analyze-146 — Handle opacity test~~ | conformance: handle не розкриває storage internals | named test + implementation-status **DONE** для B1-003 | зміна Handle API |
| 112 | **DONE** | ~~Analyze-147 — verify-on-read Object~~ | `open`/`get_by_object_id` re-verify signature; tamper fail | mutation/tamper тести | Sqlite migration; new fields |
| 113 | **DONE** | ~~Analyze-148 — verify-on-read Artifact~~ | `resolve*` re-verify descriptor + content hash | tampered sidecar/CAS → fail | new artifact types |
| 114 | **DONE** | ~~Analyze-149 — Policy Gate dispatch~~ | `CsuRuntime::dispatch` bind policy; DENY без bypass | dispatch deny тести | new policies YAML |
| 115 | **DONE** | ~~Analyze-150 — acquisition fail-closed audit~~ | regression: no ALLOW → DENY download/publish; docs | тести + docs/model layer | remote URL download |
| 116 | **DONE** | ~~Analyze-151 — split model-acquisition~~ | mechanical split `csu/model-acquisition/src/lib.rs` | `cargo test -p model-acquisition` зелений; behavior unchanged | semantic refactor |
| 117 | **DONE** | ~~Analyze-152 — C2 CI job~~ | GitHub Actions: `conformance run --profile C2` | job червоний при C2 fail; зелений на main | C2 semantic expansion |
| 118 | **DONE** | ~~Analyze-153 — flaky desktop P2 test~~ | стабілізувати `peer_lifecycle_p2` на CI | 3 consecutive CI green без flake | P3/P4 lifecycle |
| 119 | **DONE** | ~~Analyze-154 — Phase F docs + RFC~~ | README + `implementation-status.md` sync; RFC-F; close F | E4 DONE + F scope documented | Phase G planning |

### Наступний цикл (Phase F)

**Phase F завершено** (`#107`–`#119` DONE @ PR #82, RFC-0068).

```text
F0 stabilize CI (#107–#109)
  → F1 SECURITY (#110)
    → F2 verify (#111–#115)
      → F3 modularize (#116)
        → F4 harden CI (#117–#118)
          → F5 docs (#119) ✓
```

## Активна черга (лінійна) — Phase G: Reference completion v0.2

План: [`docs/phase-g-plan.md`](docs/phase-g-plan.md). Джерела: [`implementation-status.md`](docs/implementation-status.md); EVO-2 local C2→C3→CSU SDK; Phase F §3 post-F deferrals.  
**Не** CRP/settlement/PHM/HIE runtime; **не** production distributed plane.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 120 | **DONE** | ~~Analyze-155 — CI branch protection sync~~ | `conformance-c2` у required checks doc + contract test | doc/checklist узгоджені з `.github/workflows/ci.yml` | GitHub API automation |
| 121 | **DONE** | ~~Analyze-156 — Living spec smoke~~ | C0 ids у `implementation-status` ↔ `run_c0` | test/script green; missing id fails | full matrix autogen |
| 122 | **DONE** | ~~Analyze-157 — C2 event idempotency~~ | conformance `c2.event.publish_idempotent` | C2 job green; duplicate publish safe | wire network |
| 123 | **DONE** | ~~Analyze-158 — C2 artifact hash mismatch~~ | conformance `c2.artifact.hash_mismatch` | hash tamper → fail | new artifact types |
| 124 | **DONE** | ~~Analyze-159 — C2 envelope unsigned~~ | conformance `c2.protocol.envelope_unsigned` | unsigned envelope → fail | envelope semantics change |
| 125 | **DONE** | ~~Analyze-160 — Claim/Assumption fixtures~~ | valid/invalid JSON + manifest | `schema validate --fixtures` зелений | Epistemic CSU impl |
| 126 | **DONE** | ~~Analyze-161 — VRA extended fields~~ | conformance VRA field coverage | named test green | full Book I 1:1 fields |
| 127 | **DONE** | ~~Analyze-162 — split crypto mod~~ | mechanical `aira-object/src/crypto/mod.rs` | `cargo test -p aira-object` green | crypto behavior change |
| 128 | **DONE** | ~~Analyze-163 — split aira-peer lib~~ | mechanical `aira-peer/src/lib.rs` | `cargo test -p aira-peer` green | protocol change |
| 129 | **DONE** | ~~Analyze-164 — split cli peer~~ | mechanical `aira-cli/commands/peer/` | peer CLI tests green | new peer commands |
| 130 | **DONE** | ~~Analyze-165 — split desktop app~~ | mechanical `aira-desktop/src/app/` | desktop tests green | GUI redesign |
| 131 | **DONE** | ~~Analyze-166 — stabilize peer_lifecycle_p3~~ | CI-stable P3 lifecycle test | 3 consecutive CI green | P4; relay semantics |
| 132 | **DONE** | ~~Analyze-167 — stabilize peer_lifecycle_p4~~ | CI-stable P4 gossip lifecycle | 3 consecutive CI green | P5 federation GUI |
| 133 | **DONE** | ~~Analyze-168 — invite QR camera~~ | GUI camera → invite import | roundtrip smoke | non-loopback peer |
| 134 | **DONE** | ~~Analyze-182 — SEC-1 local-test trust hardening~~ | `local-test` не в runtime trust; peer reject test identity; migration note | conformance/peer negative test green | full identity ceremony |
| 135 | **DONE** | ~~Analyze-183 — SEC-2 canonical envelope signature~~ | full envelope + response canonical sign/verify; `key_ref == issuer` | C2 mutation tests per field green | wire protocol version bump |
| 136 | **DONE** | ~~Analyze-184 — SEC-3 producer identity binding~~ | `signature.key_ref == producer_identity` on Object/Event/Artifact | store + conformance reject cross-identity | delegation artifacts |
| 137 | **DONE** | ~~Analyze-185 — SEC-4 event equivocation~~ | same `event_id` + diff hash → conflict; C2 case | idempotent same-hash ACCEPT | global total order |
| 138 | **DONE** | ~~Analyze-186 — SEC-5 artifact admission~~ | immutable descriptor admission; supersession persist on reopen | artifact store tests green | new artifact types |
| 139 | **DONE** | ~~Analyze-169 — federation leave~~ | CLI `federation leave` | membership cleared; tests | Join Request protocol |
| 140 | **DONE** | ~~Analyze-170 — federation join hardening~~ | join edge cases + audit | regression tests green | remote federation |
| 141 | **DONE** | ~~Analyze-171 — C3 conformance scaffold~~ | `run_c3` minimal local cases | runner + doc; optional CI note | full C3 wire |
| 142 | **DONE** | ~~Analyze-172 — event log durability~~ | JSON log corruption recovery test | test + doc note | production event engine |
| 143 | **DONE** | ~~Analyze-173 — SQLite object path~~ | integrity/migration smoke | test + status doc | new object fields |
| 144 | **DONE** | ~~Analyze-174 — external CSU fixture~~ | third-party manifest loads | registry smoke green | CSU marketplace |
| 145 | **DONE** | ~~Analyze-175 — CSU SDK doc~~ | `docs/csu-sdk.md` + conformance case | doc + named test | SDK distribution |
| 146 | **DONE** | ~~Analyze-176 — epistemic-basic CSU~~ | `csu/epistemic-basic` EPI-001 | crate tests green | full Epistemic plane |
| 147 | **DONE** | ~~Analyze-177 — epistemic assessment path~~ | epistemic artifact roundtrip smoke | CLI/plane test | normative split CSU |
| 148 | **DONE** | ~~Analyze-178 — macOS codesign~~ | notarization script + doc update | script dry-run doc | App Store |
| 149 | **DONE** | ~~Analyze-179 — Windows MSI~~ | MSI build script + doc | artifact builds on CI runner | codesign enterprise |
| 150 | **DONE** | ~~Analyze-180 — Linux deb metapackage~~ | deb production path doc + script | package smoke | AppImage |
| 151 | **DONE** | ~~Analyze-181 — Phase G docs closure~~ | README + implementation-status v0.2; RFC-0069 | G scope documented; QUEUE G closed | post-G RFC track; анти-місія README |

### Наступний цикл (Phase G)

**Phase G завершено** (`#120`–`#151` DONE; RFC-0069). Далі — Phase H нижче.

```text
G0 govern (#120–#121)
  → G1 deepen C2 (#122–#124)
    → G2 evidence gaps (#125–#126)
      → G3 modularize (#127–#130)
        → G4 desktop peer (#131–#133)
          → G-SEC hardening (#134–#138)
            → G5 federation/C3 (#139–#141)
              → G6 durable stores (#142–#143)
                → G7 CSU SDK (#144–#145)
                  → G8 epistemic (#146–#147)
                    → G9 packaging (#148–#150)
                      → G10 docs (#151) ✓
```

## Активна черга (лінійна) — Phase H: Protocol depth & durable reference v0.3

План: [`docs/phase-h-plan.md`](docs/phase-h-plan.md). Джерела: Phase G §7 (зведені в **одну** лінію); Conformance C3→C4→C5; Book II CRP/Settlement; Book V promotion.  
**Не** GPU/LLM/blockchain/scheduler; **не** distributed OperationalPlane; CRP = capability routing; Settlement = audit receipts.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 152 | **DONE** | ~~Analyze-187 — Phase H wiring~~ | `phase-h-plan.md` + QUEUE `#152`–`#183` + cross-links; living smoke ids | docs + QUEUE узгоджені; phase_h_doc | stores code; CRP |
| 153 | **DONE** | ~~Analyze-188 — C3 governance note~~ | conformance + ci-governance: C3 optional; gate criteria | docs sync; ci_governance C3 contract | C3 CI job (#164) |
| 154 | **DONE** | ~~Analyze-189 — event hash-chain tip~~ | append + verify tip; mid-log tamper detect | `event_log_hash_chain_tip_…` green | prefix recovery (#155) |
| 155 | **DONE** | ~~Analyze-190 — event prefix recovery~~ | corrupt trailing → valid prefix recover | `corrupt_trailing_event_log_recovers_valid_prefix` | new backend (#156) |
| 156 | **DONE** | ~~Analyze-191 — durable event backend~~ | file-chain event log module | `file_chain_event_log_persists_across_reopen` | session wire (#157) |
| 157 | **DONE** | ~~Analyze-192 — session durable events~~ | LocalSession/init_node → durable backend; reopen | `session_durable_file_chain_roundtrip` | Sqlite objects (#158) |
| 158 | **DONE** | ~~Analyze-193 — Sqlite object path~~ | node/plane Sqlite object path test + doc | `plane_memory_beside_node_sqlite_object_path` | status rollup (#159) |
| 159 | **DONE** | Analyze-194 — stores status rollup | implementation-status stores/event rows + Phase H table | PARTIAL notes reflect H1 (`#154`–`#158`) | capability ad (#160) |
| 160 | **DONE** | ~~Analyze-195 — capability ad persist~~ | load/save local capability advertisements | `capability_ad_persist_roundtrip` | C3 case (#161) |
| 161 | **DONE** | ~~Analyze-196 — C3 capability case~~ | `c3.capability.advertisement` у run_c3 | case green (5 local) | policy export (#162) |
| 162 | **DONE** | ~~Analyze-197 — federation export deny~~ | policy-scoped export/import deny + audit | `federation_export_import_deny_by_default_audits` | C3 ≥6 (#163) |
| 163 | **DONE** | ~~Analyze-198 — C3 cases ≥6~~ | run_c3 ≥6 named local cases + doc | 6 cases incl. `c3.federation.export_deny` | C3 CI job (#164) |
| 164 | **DONE** | ~~Analyze-199 — optional C3 CI~~ | workflow job non-gate + governance note | `conformance-c3` in ci.yml (not required) | CRP schemas (#165) |
| 165 | **DONE** | ~~Analyze-200 — CRP schema fixtures~~ | route request/candidate schemas + fixtures | `schema validate --fixtures` green | CRP adapter (#166) |
| 166 | **DONE** | ~~Analyze-201 — CRP local adapter~~ | in-process CRP + RFC-0079; capability≠node | adapter tests + RFC | node-keyed reject (#167) |
| 167 | **DONE** | ~~Analyze-202 — CRP reject node route~~ | conformance: node-keyed route → fail | `c3.crp.reject_node_route` green | multi-candidate (#168) |
| 168 | **DONE** | ~~Analyze-203 — CRP multi-candidate gate~~ | ≥2 candidates; Policy Gate before bind | `crp_multi_candidate_and_policy_gate_bind` | CRP events (#169) |
| 169 | **DONE** | ~~Analyze-204 — CRP route events~~ | selected/rejected/failure events | `crp_route_events_selected_rejected_failure` | B2-006 case (#170) |
| 170 | **DONE** | ~~Analyze-205 — B2-006 C3 case~~ | `c3.crp.route_candidate` | run_c3 green (8 cases) | status CRP (#171) |
| 171 | **DONE** | ~~Analyze-206 — CRP status PARTIAL~~ | implementation-status CRP ABSENT→PARTIAL | matrix updated | settlement fixtures (#172) |
| 172 | **DONE** | ~~Analyze-207 — settlement fixtures~~ | receipt schema fixtures + privacy fields | `schema validate --fixtures` green | receipt store (#173) |
| 173 | **DONE** | ~~Analyze-208 — settlement receipt store~~ | append-only receipts; verify-on-read smoke | `settlement_receipt_store_*`; RFC-0080 | B2-011 (#174) |
| 174 | **DONE** | ~~Analyze-209 — B2-011 privacy smoke~~ | privacy class / redaction local test | `b2_011_settlement_privacy_smoke` | run_c4 (#175) |
| 175 | **DONE** | ~~Analyze-210 — run_c4 scaffold~~ | minimal C4 receipt cases | `run_c4` 3 cases; RFC-0081 | status settlement (#176) |
| 176 | **DONE** | ~~Analyze-211 — settlement status~~ | implementation-status Settlement→PARTIAL | Book II Settlement **PARTIAL** | RFC-P doc (#177) |
| 177 | **DONE** | ~~Analyze-212 — RFC-P promotion doc~~ | `docs/rfc-p-promotion.md` | doc merged | promotion fixtures (#178) |
| 178 | **DONE** | ~~Analyze-213 — promotion fixtures~~ | promotion-candidate schema fixtures | schema validate green | non-operational gate (#179) |
| 179 | **DONE** | ~~Analyze-214 — non-operational gate~~ | research input rejected as operational | тест | run_c5 (#180) |
| 180 | **DONE** | ~~Analyze-215 — run_c5 scaffold~~ | research separation + promotion cases | runner + doc | status rollup (#181) |
| 181 | **DONE** | ~~Analyze-216 — promotion status~~ | implementation-status C5/promotion notes | matrix updated | v0.3 docs (#182) |
| 182 | **DONE** | ~~Analyze-217 — Reference v0.3 docs~~ | README + implementation-status v0.3 | docs sync | RFC-0077 (#183) |
| 183 | **DONE** | ~~Analyze-218 — Phase H RFC-0077~~ | consolidating RFC; close H | QUEUE H DONE; no OPEN | Phase I [`docs/phase-i-plan.md`](docs/phase-i-plan.md) `#184`–`#198` |

### Наступний цикл (Phase H)

**Phase H COMPLETE** (`#152`–`#183` DONE @ RFC-0077). Далі — Phase I нижче.

```text
H0 govern (#152–#153)
  → H1 durable stores (#154–#159)
    → H2 C3 federation depth (#160–#164)
      → H3 CRP local (#165–#171)
        → H4 settlement audit (#172–#176)
          → H5 research promotion (#177–#181)
            → H6 docs RFC-0077 (#182–#183)
```

Анти-місія README **ніколи** в QUEUE.

## Архів (закрито) — Phase I: Semantic contract stabilization

План: [`docs/phase-i-plan.md`](docs/phase-i-plan.md). Джерела: аудит `main` @ `b66bcf1` (P0–P2); Phase H §7.  
**Не** GPU/LLM/blockchain/scheduler; **не** distributed OperationalPlane; **не** RFC-0078 у цьому атомі (`#198`); **не** status honesty (`#185`) у wiring.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 184 | **DONE** | ~~Analyze-219 — Phase I wiring~~ | `docs/README` + QUEUE + phase-h §7 → phase-i-plan; living smoke ids `#184`–`#198` | docs + QUEUE узгоджені; `phase_i_doc` | status honesty (#185) |
| 185 | **DONE** | ~~Analyze-220 — Status honesty rollup~~ | `implementation-status`: Opaque Handle, Reduction/reuse, Verification — **PARTIAL** (або точніше); notes з audit refs | matrix honest vs audit | Handle integrity (#186) |
| 186 | **DONE** | ~~Analyze-221 — Handle integrity~~ | `Handle::new` / `storage_token` не публічні для чужого crate; `open` bind `object_id == handle.object_ref`; adversarial tests | forge/cross-object/cross-store fail | semantic verify (#187) |
| 187 | **DONE** | ~~Analyze-222 — Semantic verify math.eval.safe~~ | VerificationBasic незалежно обчислює expression; wrong finite number → not VERIFIED | тест | PolicyGate invoke (#188) |
| 188 | **DONE** | ~~Analyze-223 — CSU PolicyGate in invoke~~ | `CsuExecutionContext` bound gate; `check_policy`; fail-closed без gate | тест | durable reuse (#189) |
| 189 | **DONE** | ~~Analyze-224 — Durable reuse index~~ | LocalSession/plane: повторний problem → reuse без execution; persistent index; e2e | тест | fail-closed signing (#190) |
| 190 | **DONE** | ~~Analyze-225 — Fail-closed signing~~ | `active_signature` без silent `local-test` fallback; demo/test явний | тест | atomic persist (#191) |
| 191 | **DONE** | ~~Analyze-226 — Atomic session persist~~ | `persist_after_submit`: temp+rename; corrupt problems index ≠ silent wipe | тест | artifact recovery (#192) |
| 192 | **DONE** | ~~Analyze-227 — Artifact metadata recovery~~ | second descriptor same content hash recoverable | тест | Clock (#193) |
| 193 | **DONE** | ~~Analyze-228 — Runtime Clock~~ | `Clock` trait; SystemClock runtime; FixedClock tests; artifacts ≠ all `2026-07-10T12:00:00Z` | тест | envelope replay (#194) |
| 194 | **DONE** | ~~Analyze-229 — Envelope freshness/replay~~ | `expires_at` / skew / message-id replay window | тести | run nonce (#195) |
| 195 | **DONE** | ~~Analyze-230 — Run nonce concurrency~~ | UUIDv7 або transactional seq замість racy `run-counter` | тест | instance crypto (#196) |
| 196 | **DONE** | ~~Analyze-231 — Instance-scoped crypto~~ | не обов’язковий process-global OnceLock signer; scoped Keyring API або doc | мінімальний тест ізоляції | MSRV CI (#197) |
| 197 | **DONE** | ~~Analyze-232 — MSRV + supply-chain CI~~ | rust-version↔CI; pin Actions SHA або documented accept; `cargo deny`/`audit` non-blocking OK | CI/doc | docs RFC (#198) |
| 198 | **DONE** | ~~Analyze-233 — Phase I docs + RFC~~ | README + implementation-status stable tag; consolidating RFC-0078; QUEUE I closed | QUEUE I DONE | — |

### Наступний цикл (Phase I)

**Phase I COMPLETE** (`#184`–`#198` DONE @ RFC-0078). QUEUE I closed. Далі — Phase J нижче.

```text
I0 govern + status honesty (#184–#185)
  → I1 P0 Core/CSU semantics (#186–#189)
    → I2 P1 identity + durability (#190–#193)
      → I3 P2 hardening (#194–#197)
        → I4 docs RFC-0078 (#198)
```

Анти-місія README **ніколи** в QUEUE.

## Активна черга (лінійна) — Phase J: Book-gap local remainder

План: [`docs/phase-j-plan.md`](docs/phase-j-plan.md). Джерела: матриця Book 0–IV після RFC-0078; [`docs/phase-i-plan.md`](docs/phase-i-plan.md) §0.  
**Не** GPU/LLM/blockchain/scheduler; **не** Book II wire / distributed OperationalPlane; consolidating RFC-0096 у `#208`; **не** Book II honesty (`#200`) у wiring.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 199 | **DONE** | ~~Analyze-234 — Phase J wiring~~ | `docs/README` + QUEUE + phase-i §0/§9 → phase-j-plan; living smoke ids `#199`–`#208` | docs + QUEUE узгоджені; `phase_j_doc` | Book II honesty (#200) |
| 200 | **DONE** | ~~Analyze-235 — Book II ceiling honesty~~ | матриця envelope/EP/AP/identity/discovery/CAP/CRP/settlement: **PARTIAL** + «local adapter = v0.3 ceiling» | matrix honest vs Book II | Handle API (#201) |
| 201 | **DONE** | ~~Analyze-236 — Seal `object_store_access`~~ | `mint` не в public prelude `aira-object` для CSU; store-backend лише `aira-core` | тест | VRA payload (#202) |
| 202 | **DONE** | ~~Analyze-237 — VRA runtime B1-010~~ | C1 `calculate_2_plus_2` artifact body містить усі `required` з `schemas/result/verified-result-artifact.schema.json` | тест | event-log (#203) |
| 203 | **DONE** | ~~Analyze-238 — Event-log authority~~ | після persist reopen `event_tail` читає `events/file-chain-log.json`; memory `drain_from` не єдине джерело | тест | catalog (#204) |
| 204 | **DONE** | ~~Analyze-239 — Reduction catalog bind~~ | plane Reduction бачить durable reuse без ручного `enable_ready_solution` | тест | text.verify (#205) |
| 205 | **DONE** | ~~Analyze-240 — Semantic verify text.*~~ | `text.echo` / `text.uppercase`: wrong string → not VERIFIED | тест | evidence (#206) |
| 206 | **DONE** | ~~Analyze-241 — Evidence primacy runtime~~ | Claim vs Assumption reject у runtime (не лише fixtures `#125`) | тест | epistemic (#207) |
| 207 | **DONE** | ~~Analyze-242 — Epistemic emit on C1~~ | C1 2+2 path пише epistemic-assessment artifact (не full Epistemic plane) | тест | RFC (#208) |
| 208 | **DONE** | ~~Analyze-243 — Phase J docs + RFC~~ | README + implementation-status **v0.3-strict**; consolidating RFC-0096; QUEUE J closed | QUEUE J DONE | — |

### Наступний цикл (Phase J)

**Phase J COMPLETE** (`#199`–`#208` DONE @ RFC-0096). QUEUE J closed. no OPEN J atoms. Next: Phase K [`docs/phase-k-plan.md`](docs/phase-k-plan.md) `#209`–`#216` **DONE**. Phase L [`docs/phase-l-plan.md`](docs/phase-l-plan.md) `#217`–`#223` **DONE** @ RFC-0111. QUEUE L closed.

`#199` wiring **DONE**. `#200` Book II ceiling honesty **DONE**. `#201` Seal `object_store_access` **DONE**. `#202` VRA runtime B1-010 **DONE**. `#203` Event-log authority **DONE**. `#204` Reduction catalog bind **DONE**. `#205` Semantic verify text.* **DONE**. `#206` Evidence primacy runtime **DONE**. `#207` Epistemic emit on C1 **DONE**. `#208` docs + RFC-0096 **DONE**.

```text
J0 govern + Book II ceiling honesty (#199–#200)
  → J1 Book I remainder (#201–#203)
    → J2 Book 0/III remainder (#204–#207)
      → J3 docs RFC-0096 (#208)
```

Анти-місія README **ніколи** в QUEUE.

---

## Активна черга (лінійна) — Phase K: Local LLM Execution CSU

План: [`docs/phase-k-plan.md`](docs/phase-k-plan.md). Джерела: Book I §2 LLM Backend CSU; EVO-3 §8–§10; Phase D inventory/activate **DONE**; рішення розробника 2026-09-01.  
**Не** GPU marketplace; **не** LLM у `aira-core`; consolidating RFC-0104 у `#216`; **не** capsule schema (`#210`) у wiring.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 209 | **DONE** | ~~Analyze-244 — Phase K wiring~~ | `docs/README` + QUEUE + phase-j §9 + phase-d §6 → phase-k-plan; living smoke ids `#209`–`#216` | docs + QUEUE узгоджені; `phase_k_doc` | capsule schema (#210) |
| 210 | **DONE** | ~~Analyze-245 — Capsule `text.generate.local`~~ | payload schema `aira:schema:execution:generate-local:0.1` + fixtures; RFC-S (RFC-0105) | schema validate --fixtures | execution-llm crate (#211) |
| 211 | **DONE** | ~~Analyze-246 — `execution-llm` CSU + mock~~ | CapsuleCreated → CapsuleCompleted via MockBackend; fail-closed without backend | тести CSU; не в aira-core | Reduction (#212); ollama (#215) |
| 212 | **DONE** | ~~Analyze-247 — Reduction bind~~ | non-math/echo/upper → `text.generate.local`; `Calculate 2 + 2` лишається `math.eval.safe` | тест 2+2 не зламаний | plane register (#213) |
| 213 | **DONE** | ~~Analyze-248 — Plane register~~ | OperationalPlane/LocalSession диспатчить generate на execution-llm | mock completes in tests | activate gate (#214) |
| 214 | **DONE** | ~~Analyze-249 — Activate gate~~ | generate без Phase D activate → CapsuleFailed + Evidence, не VERIFIED | тест fail-closed | process backend (#215) |
| 215 | **DONE** | ~~Analyze-250 — Process backend~~ | argv/local CLI adapter; missing binary fail-closed; CI лишає mock | тест missing-binary | Desktop RFC (#216) |
| 216 | **DONE** | ~~Analyze-251 — Desktop + RFC-0104~~ | Work tab generate path; consolidating RFC-0104; QUEUE K closed | QUEUE K DONE | — |

### Наступний цикл (Phase K)

**Phase K COMPLETE** (`#209`–`#216` DONE @ RFC-0104). QUEUE K closed. no OPEN K atoms. Next: Phase L [`docs/phase-l-plan.md`](docs/phase-l-plan.md) `#217`–`#223` **DONE** @ RFC-0111. QUEUE L closed.

`#209` wiring **DONE**. `#210` generate-local schema **DONE** (RFC-0105). `#211` `execution-llm` CSU + mock **DONE** (RFC-0106). `#212` Reduction bind **DONE** (RFC-0107). `#213` Plane register **DONE** (RFC-0108). `#214` Activate gate **DONE** (RFC-0109). `#215` Process backend **DONE** (RFC-0110). `#216` Desktop + RFC-0104 **DONE**.

```text
K0 govern (#209)
  → K1 capsule (#210)
    → K2 execution-llm mock (#211)
      → K3–K4 Reduction + plane (#212–#213)
        → K5 activate gate (#214)
          → K6 process backend (#215)
            → K7 Desktop RFC-0104 (#216)
```

Анти-місія README **ніколи** в QUEUE.

---

## Активна черга (лінійна) — Phase L: Execution-boundary hardening

План: [`docs/phase-l-plan.md`](docs/phase-l-plan.md). Джерела: post-K audit 2026-09-03 (activate pointer-exists; child env; unbounded pipes; ProblemRecord verified lie; `network=none` not OS-sandbox).  
**Не** GPU marketplace; **не** LLM у `aira-core`; **не** Landlock/seccomp у L; consolidating RFC-0111 у `#223` **DONE**; **не** activate evidence (`#218`) у wiring.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 217 | **DONE** | ~~Analyze-252 — Phase L wiring~~ | `docs/README` + QUEUE + phase-k + NEXT_PROBLEM → phase-l-plan; living smoke ids `#217`–`#223`; L0 govern | docs + QUEUE узгоджені; `phase_l_doc` | Activate evidence (#218) |
| 218 | **DONE** | ~~Analyze-253 — Activate evidence~~ | gate: `content_hash` + cache bytes + evidence artifact; forged `{model_ref}` fail-closed | тести fail-closed | env (#219) |
| 219 | **DONE** | ~~Analyze-254 — Child env whitelist~~ | ProcessBackend `env_clear` + PATH/HOME/LANG; no `AIRA_HTTP_TOKEN` | тест whitelist | bounded IO (#220) |
| 220 | **DONE** | ~~Analyze-255 — Bounded pipes~~ | stdout/stderr cap **during** read; overflow → CapsuleFailed | тест overflow | ProblemRecord (#221) |
| 221 | **DONE** | ~~Analyze-256 — ProblemRecord split~~ | persisted `execution_artifact_id`; GET не кладе executed в `verified_artifact_id` | GET/POST parity | network RFC (#222) |
| 222 | **DONE** | ~~Analyze-257 — `network=none` contract~~ | RFC: AIRA-mediated none; not OS sandbox | RFC + docs | RFC-0111 (#223) |
| 223 | **DONE** | ~~Analyze-258 — Phase L docs + RFC-0111~~ | consolidating RFC-0111; QUEUE L closed | QUEUE L DONE | — |

### Наступний цикл (Phase L)

**Phase L COMPLETE** (`#217`–`#223` DONE @ RFC-0111). QUEUE L closed. no OPEN L atoms. A next phase is not activated here.

`#217` L0 govern **DONE**. `#218` Activate evidence **DONE** (RFC-0112). `#219` Child env whitelist **DONE** (RFC-0113). `#220` Bounded pipes **DONE** (RFC-0114). `#221` ProblemRecord split **DONE** (RFC-0115). `#222` `network=none` contract **DONE** (RFC-0116). `#223` RFC-0111 + close **DONE**.

```text
L0 govern (#217)
  → L1 Activate evidence (#218)
    → L2 env whitelist (#219)
      → L3 bounded IO (#220)
        → L4 ProblemRecord (#221)
          → L5 network contract (#222)
            → L6 RFC-0111 (#223)
```

Анти-місія README **ніколи** в QUEUE.

---

## Активна черга (лінійна) — Phase M: OS child sandbox

План: [`docs/phase-m-plan.md`](docs/phase-m-plan.md). Джерела: Phase L §3.1 Out; RFC-0116 post-L Landlock/seccomp/netns.  
**Не** GPU marketplace; **не** LLM у `aira-core`; consolidating RFC-0117 у `#230` **DONE**; `#225` Landlock **DONE**; `#226` seccomp **DONE**; `#227` netns **DONE**; `#228` sandbox-required **DONE**; `#229` OS vs AIRA-mediated **DONE**.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 224 | **DONE** | ~~Analyze-259 — Phase M wiring~~ | `docs/README` + QUEUE + phase-l + NEXT_PROBLEM → phase-m-plan; living smoke ids `#224`–`#230`; M0 govern | docs + QUEUE узгоджені; `phase_m_doc` | Landlock (#225) |
| 225 | **DONE** | ~~Analyze-260 — Landlock FS~~ | Linux Landlock on ProcessBackend child; FS restriction fail-closed | тест Landlock | seccomp (#226) |
| 226 | **DONE** | ~~Analyze-261 — seccomp~~ | syscall filter on child; forbidden syscall → CapsuleFailed | тест seccomp | netns (#227) |
| 227 | **DONE** | ~~Analyze-262 — netns~~ | Linux network namespace for offline argv; ollama loopback not silently broken | тест netns | policy (#228) |
| 228 | **DONE** | ~~Analyze-263 — missing sandbox fail-closed~~ | sandbox required + no kernel/OS → CapsuleFailed, not VERIFIED | тест fail-closed | schema (#229) |
| 229 | **DONE** | ~~Analyze-264 — OS vs AIRA-mediated docs~~ | schema/docs distinguish OS isolation from RFC-0116 adapter none | RFC + docs | RFC-0117 (#230) |
| 230 | **DONE** | ~~Analyze-265 — Phase M docs + RFC-0117~~ | consolidating RFC-0117; QUEUE M closed | QUEUE M DONE | — |

### Наступний цикл (Phase M)

**Phase M COMPLETE** (`#224`–`#230` DONE @ RFC-0117). QUEUE M closed. no OPEN M atoms. Next: Phase N [`docs/phase-n-plan.md`](docs/phase-n-plan.md) `#231`–`#247`.

`#224` M0 govern **DONE**. `#225` Landlock **DONE** (RFC-0118). `#226` seccomp **DONE** (RFC-0119). `#227` netns **DONE** (RFC-0120). `#228` sandbox-required **DONE** (RFC-0121). `#229` OS vs AIRA-mediated **DONE** (RFC-0122). `#230` RFC-0117 + close **DONE**.

```text
M0 govern (#224)
  → M1 Landlock FS (#225)
    → M2 seccomp (#226)
      → M3 netns (#227)
        → M4 fail-closed policy (#228)
          → M5 OS vs AIRA-mediated docs (#229)
            → M6 RFC-0117 (#230)
```

Анти-місія README **ніколи** в QUEUE.

---

## Активна черга (лінійна) — Phase N: Global Node Rendezvous & Prime Connectivity

План: [`docs/phase-n-plan.md`](docs/phase-n-plan.md). Джерела: `aira-prime.md` + `aira-current.md` (2026-09-05); QUEUE M closed @ RFC-0117.  
**Не** GPU marketplace; **не** LLM у `aira-core`; **не** AIRA-owned consensus/tokenomics; consolidating RFC-0123 у `#247`; **не** Prime Port (`#232`) у wiring.

| # | Status | Analyze | Атомарний scope | Done when | Не в цьому рядку |
|---|--------|---------|-----------------|-----------|------------------|
| 231 | **DONE** | ~~Analyze-266 — Phase N wiring~~ | `docs/README` + QUEUE + phase-m + NEXT_PROBLEM → phase-n-plan; living smoke ids `#231`–`#247`; N0 govern | docs + QUEUE узгоджені; `phase_n_doc` | Prime Port (#232) |
| 232 | **DONE** | ~~Analyze-267 — Prime Port invariant~~ | `aira-peer` `prime_port`; P_AIRA count 1491; fail-closed non-prime AIRA endpoints | тести + RFC-0124 | preferred selection (#233) |
| 233 | **DONE** | ~~Analyze-268 — Deterministic port selection~~ | preferred_port(identity, class); collision → next; wrap finite | тести + RFC-0125 | Presence (#234) |
| 234 | **OPEN** | Analyze-269 — Presence Record | schema + canonical Ed25519; mutation tests | schema + тести | Rendezvous trait (#235) |
| 235 | **OPEN** | Analyze-270 — RendezvousProvider | trait API; no ledger logic in Core | trait + mock | EVM adapter (#236) |
| 236 | **OPEN** | Analyze-271 — EVM ledger adapter | local deterministic double; Amoy/mainnet config hooks | adapter + тести | publish/query (#237) |
| 237 | **OPEN** | Analyze-272 — Publish/query | publish/update/query_active/query_identity; TTL/sequence | тести | Reachability probe (#238) |
| 238 | **OPEN** | Analyze-273 — Reachability Probe | peer-assisted signed challenge; no hairpin proof | тести | state machine (#239) |
| 239 | **OPEN** | Analyze-274 — Reachability states | UNKNOWN…OFFLINE; `reachability.json` | тести | AddressBook (#240) |
| 240 | **OPEN** | Analyze-275 — AddressBook promotion | valid Presence + trust policy → AddressBook only | тести | relay (#241) |
| 241 | **OPEN** | Analyze-276 — Relay integration | direct→NAT→relay; prime relay ads | тести | refresh (#242) |
| 242 | **OPEN** | Analyze-277 — Presence refresh | sequence++; expire stale; endpoint change | тести | CLI (#243) |
| 243 | **OPEN** | Analyze-278 — CLI | `peer port` / `reachability` / `rendezvous` commands | CLI + тести | Desktop (#244) |
| 244 | **OPEN** | Analyze-279 — Desktop UX | Network: port/reachability/rendezvous/peer count | GUI + тести | ab ovo (#245) |
| 245 | **OPEN** | Analyze-280 — Ab ovo integration | A publish → B discover → trust → dial; no preconfigured peers | integration | NAT/relay (#246) |
| 246 | **OPEN** | Analyze-281 — NAT/relay integration | both inbound blocked → relay Noise succeeds | integration | RFC close (#247) |
| 247 | **OPEN** | Analyze-282 — Phase N docs + RFC-0123 | consolidating RFC-0123; QUEUE N closed | QUEUE N DONE | — |

### Наступний цикл (Phase N)

**Перший OPEN:** `#234` (канон: перший OPEN `#234`). `#231`–`#233` **DONE** (RFC-0124/0125; N2 selection). RFC-0123 file-free until `#247`.

```text
N0 govern (#231)
  → N1 Prime Port (#232) DONE
    → N2 selection (#233) DONE
      → N3 Presence (#234)
        → N4–N6 Rendezvous (#235–#237)
          → N7–N8 Reachability (#238–#239)
            → N9–N11 mesh glue (#240–#242)
              → N12–N13 UX (#243–#244)
                → N14–N15 integration (#245–#246)
                  → N16 RFC-0123 (#247)
```

Анти-місія README **ніколи** в QUEUE.

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
| EVO-3 model layer (CSU/Artifact, не Core) | план [`docs/phase-d-plan.md`](docs/phase-d-plan.md); `#53`–`#74` DONE (D0–D7 complete) |
| Desktop one-click / UX (розробник 2026-08-20) | [`NEXT_PROBLEM.md`](NEXT_PROBLEM.md) RESOLVED → [`docs/desktop-ux.md`](docs/desktop-ux.md); [`docs/phase-e-plan.md`](docs/phase-e-plan.md); `#75`–`#106` DONE (2026-08-22) |
| Post-E stabilization (phase-e §Пізніше; EVO-2) | [`docs/phase-f-plan.md`](docs/phase-f-plan.md); `#107`–`#119` **DONE** (2026-08-24) |
| Reference v0.2 (local C2 depth, SEC hardening, C3 scaffold, SDK, packaging) | [`docs/phase-g-plan.md`](docs/phase-g-plan.md); `#120`–`#151` **DONE** @ RFC-0069 |
| Protocol depth v0.3 | [`docs/phase-h-plan.md`](docs/phase-h-plan.md); `#152`–`#183` **DONE** @ RFC-0077 |
| Semantic contract stabilization (post-H) | [`docs/phase-i-plan.md`](docs/phase-i-plan.md); `#184`–`#198` **DONE** @ RFC-0078 |
| Book-gap local remainder (post-I) | [`docs/phase-j-plan.md`](docs/phase-j-plan.md); `#199`–`#208` **DONE** @ RFC-0096; QUEUE J closed |
| Local LLM Execution CSU (post-J / post-D) | [`docs/phase-k-plan.md`](docs/phase-k-plan.md); `#209`–`#216` **DONE** @ RFC-0104; QUEUE K closed |
| Execution-boundary hardening (post-K audit) | [`docs/phase-l-plan.md`](docs/phase-l-plan.md); `#217`–`#223` **DONE** @ RFC-0111; QUEUE L closed |
| OS child sandbox (post-L) | [`docs/phase-m-plan.md`](docs/phase-m-plan.md); `#224`–`#230` **DONE** @ RFC-0117; QUEUE M closed |
| Global rendezvous & prime connectivity (post-M) | [`docs/phase-n-plan.md`](docs/phase-n-plan.md); `#231`–`#233` **DONE**; `#234`–`#247` OPEN; RFC-0124/0125; RFC-0123 reserved |

Після DONE рядка: позначити `~~…~~ **DONE**`, оновити «Наступний цикл», закрити відповідний `analysis/Analyze-N/`.
