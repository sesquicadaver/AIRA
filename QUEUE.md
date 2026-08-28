# AIRA — лінійна черга задач

**Оновлено:** 2026-08-28  
**Правило виконання:** завжди береться **перший OPEN** рядок; один рядок = один Analyze-цикл = одна атомарна зміна; не пропускати; не зливати два рядки в один PR; не чіпати `Manifesto etc/**`, `Meditation_About/**`.  
**Канон:** цей файл. Старі `analysis/Analyze-*/todo/TODO_FIXME.md` — лише provenance.  
**План етапу:** Phase C [`docs/phase-c-plan.md`](docs/phase-c-plan.md) **DONE**. Phase D [`docs/phase-d-plan.md`](docs/phase-d-plan.md) — D0–D7 (`#53`–`#74`) **DONE**. Phase E [`docs/phase-e-plan.md`](docs/phase-e-plan.md) + UX [`docs/desktop-ux.md`](docs/desktop-ux.md) — `#75`–`#106` **DONE** (2026-08-22). Phase F Post-E stabilization [`docs/phase-f-plan.md`](docs/phase-f-plan.md) — `#107`–`#119` **DONE** (2026-08-24). Phase G Reference v0.2 [`docs/phase-g-plan.md`](docs/phase-g-plan.md) — `#120`–`#151` **DONE** (2026-08-28, RFC-0069). Phase H Protocol depth v0.3 [`docs/phase-h-plan.md`](docs/phase-h-plan.md) — `#152`–`#183` **OPEN**.

## Стан

| | |
|--|--|
| `main` | Phase G `#120`–`#151` **DONE** @ RFC-0069; Phase H `#152`–`#155` **DONE**; `#156` **OPEN** |
| MVP / Peer P0–P2 micros #1–17 | **архів (DONE)** |
| Phase B #18–#37 | **архів (DONE)** |
| Активна черга | Phase H [`docs/phase-h-plan.md`](docs/phase-h-plan.md) — перший OPEN `#156` |

## Правила атомарності

1. **Лінійність:** `#N` стартує лише після `#N-1` = DONE (APPROVE/CLEAR + UltraQA + push).
2. **Один вихід:** кожен рядок має один measurable «Done when».
3. **Не в scope:** колонка обов’язкова — усе інше відкладається в наступні рядки.
4. **Anti-merge:** Noise+NAT+DHT / dual-key+Noise / authn+federation — заборонені в одному рядку. Phase C: не зливати типи дескрипторів, не зливати split різних `.rs`, не зливати CI з crypto. Phase D: не зливати різні payload schema files; не зливати schema з downloader.
5. **Перенумерація:** нові задачі лише **в кінець** OPEN-хвоста; не вставляти між DONE і поточним OPEN без окремого рішення розробника.
6. **План-перед-чергою:** поодинокі пункти не додавати. Phase C `#38`–`#52` DONE. Phase D `#53`–`#74` DONE. Phase E `#75`–`#106` DONE. Phase F `#107`–`#119` DONE. Phase G — [`docs/phase-g-plan.md`](docs/phase-g-plan.md); `#120`–`#151` **DONE** (2026-08-28). Phase H — [`docs/phase-h-plan.md`](docs/phase-h-plan.md); `#152`–`#183` **OPEN**.

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
| 156 | **OPEN** | Analyze-191 — durable event backend | file-chain або SQLite events module | unit tests green | session wire (#157) |
| 157 | **OPEN** | Analyze-192 — session durable events | LocalSession/init_node → durable backend; reopen | roundtrip test | Sqlite objects (#158) |
| 158 | **OPEN** | Analyze-193 — Sqlite object path | node/plane Sqlite object path test + doc | test green; no Core→node | status rollup (#159) |
| 159 | **OPEN** | Analyze-194 — stores status rollup | implementation-status stores/event rows | PARTIAL notes reflect H1 | capability ad (#160) |
| 160 | **OPEN** | Analyze-195 — capability ad persist | load/save local capability advertisements | тест | C3 case (#161) |
| 161 | **OPEN** | Analyze-196 — C3 capability case | `c3.capability.advertisement` у run_c3 | case green | policy export (#162) |
| 162 | **OPEN** | Analyze-197 — federation export deny | policy-scoped export/import deny + audit | smoke green | C3 ≥6 (#163) |
| 163 | **OPEN** | Analyze-198 — C3 cases ≥6 | run_c3 ≥6 named local cases + doc | suite count met | C3 CI job (#164) |
| 164 | **OPEN** | Analyze-199 — optional C3 CI | workflow job non-gate + governance note | job runs on PR/main | CRP schemas (#165) |
| 165 | **OPEN** | Analyze-200 — CRP schema fixtures | route request/candidate fixtures validate | schema CI green | CRP adapter (#166) |
| 166 | **OPEN** | Analyze-201 — CRP local adapter | in-process CRP + short RFC; capability≠node | adapter tests + RFC | node-keyed reject (#167) |
| 167 | **OPEN** | Analyze-202 — CRP reject node route | conformance: node-keyed route → fail | named case green | multi-candidate (#168) |
| 168 | **OPEN** | Analyze-203 — CRP multi-candidate gate | ≥2 candidates; Policy Gate before bind | ALLOW/DENY tests | CRP events (#169) |
| 169 | **OPEN** | Analyze-204 — CRP route events | selected/rejected/failure events | тест | B2-006 case (#170) |
| 170 | **OPEN** | Analyze-205 — B2-006 C3 case | `c3.crp.route_candidate` | run_c3 green | status CRP (#171) |
| 171 | **OPEN** | Analyze-206 — CRP status PARTIAL | implementation-status CRP ABSENT→PARTIAL | matrix updated | settlement fixtures (#172) |
| 172 | **OPEN** | Analyze-207 — settlement fixtures | receipt schema fixtures + privacy fields | schema validate green | receipt store (#173) |
| 173 | **OPEN** | Analyze-208 — settlement receipt store | append-only receipts; verify-on-read smoke | tests green | B2-011 (#174) |
| 174 | **OPEN** | Analyze-209 — B2-011 privacy smoke | privacy class / redaction local test | named test | run_c4 (#175) |
| 175 | **OPEN** | Analyze-210 — run_c4 scaffold | minimal C4 receipt cases | runner + doc | status settlement (#176) |
| 176 | **OPEN** | Analyze-211 — settlement status | implementation-status Settlement→PARTIAL | matrix updated | RFC-P doc (#177) |
| 177 | **OPEN** | Analyze-212 — RFC-P promotion doc | `docs/rfc-p-promotion.md` | doc merged | promotion fixtures (#178) |
| 178 | **OPEN** | Analyze-213 — promotion fixtures | promotion-candidate schema fixtures | schema validate green | non-operational gate (#179) |
| 179 | **OPEN** | Analyze-214 — non-operational gate | research input rejected as operational | тест | run_c5 (#180) |
| 180 | **OPEN** | Analyze-215 — run_c5 scaffold | research separation + promotion cases | runner + doc | status rollup (#181) |
| 181 | **OPEN** | Analyze-216 — promotion status | implementation-status C5/promotion notes | matrix updated | v0.3 docs (#182) |
| 182 | **OPEN** | Analyze-217 — Reference v0.3 docs | README + implementation-status v0.3 | docs sync | RFC-0077 (#183) |
| 183 | **OPEN** | Analyze-218 — Phase H RFC-0077 | consolidating RFC; close H | QUEUE H DONE; no OPEN | post-H new plan only |

### Наступний цикл (Phase H)

**Перший OPEN:** `#156` (durable event backend). `#152`–`#155` DONE. Далі строго лінійно `#157`…`#183`.

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
| Reference v0.2 (local C2 depth, SEC hardening, C3 scaffold, SDK, packaging) | [`docs/phase-g-plan.md`](docs/phase-g-plan.md); `#120`–`#151` OPEN (2026-08-26) |

Після DONE рядка: позначити `~~…~~ **DONE**`, оновити «Наступний цикл», закрити відповідний `analysis/Analyze-N/`.
