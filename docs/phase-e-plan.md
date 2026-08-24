# Phase E — Desktop UX & One-Click Start v0.1

**Статус:** складено 2026-08-20; **Linux E1 (`#75`–`#79`) DONE**; **E1.1 (`#80`–`#85`) DONE** @ 2026-08-21; **Addendum E2 macOS (`#86`–`#89`) DONE** @ 2026-08-22; **Addendum E3 Windows (`#90`–`#93`) DONE** @ 2026-08-22; **Addendum E4 network profiles P2–P6 (`#94`–`#106`) DONE** @ 2026-08-22.  
**Рішення UX:** [`desktop-ux.md`](desktop-ux.md). Provenance проблеми: [`NEXT_PROBLEM.md`](../NEXT_PROBLEM.md) (**RESOLVED**).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md). Поза цим планом / addendum поодинокі пункти в QUEUE не додавати.  
**Не канон Book:** не змінює Core / C0–C1 / OperationalPlane semantics.  
**Product posture:** Linux E1 = **Developer Preview** shell над reference local plane (не production distributed AIRA). Stabilization (Handle, verify-on-read, CI branch protection, acquisition fail-closed, C2 CI, …) — Phase F **DONE** [`phase-f-plan.md`](phase-f-plan.md) `#107`–`#119`.

## 0. Порядок

```text
NEXT_PROBLEM.md (RESOLVED provenance)
  → desktop-ux.md (рішення)
    → цей план
      → лінійний хвіст QUEUE #75+
        → один OPEN = один Analyze-цикл
```

ОС: **Linux → macOS → Windows** по черзі.

Порядок поставки: **`#76` → `#77` → `#78` → `#79`** (orchestrator → launcher → GUI → package). Не пакетувати до GUI.

## 1. Класифікація

```text
Phase E = product shell over existing local plane
  + settings schema
  + shared lifecycle library (CLI + GUI)
  + Desktop GUI/tray (end-user)
  + Dev edition keeps full CLI
```

**Не** Phase E: новий federation mode, публічний bind default, marketplace, операторський runbook замість Desktop, окрема stabilization phase *перед* `#75`.

## 2. Інваріанти

1. End-user старт = GUI; Developer = GUI опційно + CLI.
2. E1 Linux = **лише P0** (local HTTP). P1+ — Addendum E1.1 (файл/QR).
3. `open_ui_on_start` / `autostart_on_login` — settings; autostart default **off**.
4. HTTP loopback завжди в Desktop; peer не стартує в E1.
5. C1 / `aira-core` / Book 0 — **не змінювати** заради Desktop.
6. Packaging macOS/Windows — лише після DONE Linux E1 (Addendum E2/E3).
7. CLI surface: **`aira desktop start|stop|status`** (не `aira start` / не окремий `aira-desktop` binary-name як канон).
8. GUI tech: **Rust-only** tray/status (egui/native або еквівалент). **Без** Node.js / окремого web frontend build dependency. Tauri — не default.
9. Lifecycle logic — **одна** shared library (напр. внутрішній crate / модуль `aira-desktop-runtime`); tray **не** shell-ить CLI; CLI **не** дублює process-management недоступний GUI.
10. E1 = Developer Preview: чесно в docs; не позиціонувати як production distributed AIRA.

### 2.1 Product data layout (норматив для `#75`/`#76`)

| Роль | Desktop | Dev |
|------|---------|-----|
| Node root | OS application-data (Linux `~/.local/share/aira/`; macOS `~/Library/Application Support/AIRA/`; Windows `%LOCALAPPDATA%\AIRA\`) | explicit `--root` (default `.aira` / CLI convention) |
| Settings | OS configuration (Linux `~/.config/aira/…`; macOS `~/Library/Preferences/AIRA/…`; Windows `%APPDATA%\AIRA\…`) | той самий schema; path через flag або colocated |
| PID / lock | OS runtime (Linux XDG state/runtime; macOS `…/AIRA/runtime`; Windows `%LOCALAPPDATA%\AIRA\runtime`) | under root або runtime dir |
| Logs | OS log/cache (Linux cache; macOS `~/Library/Logs/AIRA`; Windows `%LOCALAPPDATA%\AIRA\logs`) | under root or log dir |

Schema `#75` фіксує поля; оркестратор `#76` реалізує шляхи.

### 2.2 Lifecycle contract (обов’язковий acceptance `#76`, використовується `#78`)

```text
idempotent start;
single-instance lock;
PID + process identity validation;
stale PID recovery;
readiness через GET /health;
graceful shutdown;
bounded SIGTERM → kill fallback;
child process-group cleanup;
crash detection;
durable bounded logs;
status ∈ { stopped, starting, running, unhealthy, stopping, failed }.
```

### 2.3 Політика порту (обов’язковий acceptance `#76`)

```text
порт вільний
  → старт;

порт зайнятий сумісним AIRA instance (той самий instance_id + root)
  → attach, не другий процес;

порт зайнятий іншим процесом / іншим profile
  → fail-closed із точним повідомленням;

зміна порту
  → лише через settings (#75 поле), без прихованого auto-increment.
```

### 2.4 Local HTTP security (обов’язковий acceptance `#75` schema + `#76`/`#78` runtime)

Loopback ≠ authorization boundary. Unauthenticated P0 не повинен експонувати mutating management routes без контракту.

Один із двох (обрати в `#76`, зафіксувати в settings `#75`):

1. **Per-install token** — генерується при first run; GUI/CLI передає `Authorization: Bearer`; token у settings/runtime secrets з mode 0600; **або**
2. **Вузький Desktop IPC/API** — небезпечні management routes не в unauthenticated loopback surface.

«Open UI» у E1 = **нативний status/settings window** (і/або authenticated API), **не** голий браузер на JSON `/v1/*` як єдиний UX.

### 2.5 Autostart ownership

- `#76`: **не** встановлює OS login hooks (лише читає setting).
- `#78`: **реалізує** enable/disable OS autostart (Linux XDG autostart `.desktop`) відповідно до `autostart_on_login`.
- `#79`: пакує артефакти так, щоб `#78` hooks вказували на встановлені шляхи.

## 3. Атоми → QUEUE `#75`–`#79` (перша хвиля E0+E1 Linux P0)

| ID | Підфаза | Атом | Done when | Не в цьому рядку |
|----|---------|------|-----------|------------------|
| `#75` | E0.1 | Desktop settings schema | `aira:schema:desktop:settings:0.1` + fixtures; поля: `network_profile`(=P0), `open_ui_on_start`, `autostart_on_login`, `http_listen`, `instance_id` (або екв.), auth-mode/token-ref placeholders для §2.4; validate --fixtures | orchestrator; GUI; P1+; реальна генерація token |
| `#76` | E0.2 | Shared lifecycle + CLI `aira desktop …` | P0: layout §2.1; init+identity; start/stop/status за §2.2–§2.3; HTTP security контракт §2.4 (token **або** вузький API); тести lifecycle (idempotent, stale PID, port conflict, attach) | tray; AppImage; P1; OS autostart hooks |
| `#77` | E1.1 | Linux `.desktop` launcher | `.desktop` → desktop start (shared lib/CLI); stop документовано (CLI/tray); без peer | tray UI; AppImage; macOS/Win |
| `#78` | E1.2 | Linux tray/GUI (Rust-only) | Status + Open UI (native) + Settings + Quit; старт через shared lib; persist settings; **OS autostart hooks** §2.5 | P1; AppImage; Tauri/web stack; інші ОС |
| `#79` | E1.3 | Linux packaging | AppImage **або** tarball+`.desktop` + docs (Developer Preview); Dev CLI notes; шляхи сумісні з `#78` autostart | macOS DMG; Windows installer; P1 |

```text
#75 settings schema
  → #76 shared lifecycle + aira desktop CLI (P0)
    → #77 Linux .desktop
      → #78 Linux tray/GUI + autostart hooks
        → #79 Linux package
```

## 4. Відкладені / відкриті addendum

### 4a. Addendum E1.1 — P1 + онбординг друга (файл/QR) — **DONE**

**Рішення розробника:** 2026-08-20 — відкрити **лише E1.1** після `#79` DONE. E2/E3 / P2–P6 не копіювати в QUEUE.

**Інваріанти E1.1 (додатково до §2)**

1. `network_profile=P1` дозволений у Desktop runtime; **P2–P6** лишаються fail-closed.
2. P1 = HTTP loopback (`aira-node --http`) **+** окремий supervised `peer listen --recv` (другий процес). Без DHT / relay / gossip / federation.
3. **`peer_listen` обов’язковий при P1.** Default: `127.0.0.1:9797` (same-host Dev Preview). Non-loopback — лише якщо користувач явно задав `peer_listen` (існуюча peer-семантика `listen_explicit`); **ніколи** Desktop `--allow-public-bind`.
4. Онбординг друга = обмін **PeerInvite** (pubkey / identity_ref + опційний dial `addr`) через **файл** і **QR (PNG encode + decode з файлу зображення)**. Камера / live scan — **Out**.
5. Import invite → `trust add` + address-book upsert (peer уже trusted); без авто-trust невідомих з мережі.
6. C1 / `aira-core` / Book 0 — **не змінювати**. Settings `$id` лишається `0.1` (enum уже містить P1).
7. GUI tech без змін: Rust-only egui.

**Атоми → QUEUE `#80`–`#85`** (Analyze-115+)

| ID | Підфаза | Атом | Done when | Не в цьому рядку |
|----|---------|------|-----------|------------------|
| `#80` | E1.1.0 | PeerInvite payload schema | `aira:schema:desktop:peer-invite:0.1` + fixtures; identity_ref/pubkey + optional addr | settings P1; peer process; QR; GUI |
| `#81` | E1.1.1 | Settings: P1 + `peer_listen` | runtime приймає P1; default/validate `peer_listen`; fixtures/docs; P2+ fail-closed | peer supervise; invite IO; GUI |
| `#82` | E1.1.2 | Lifecycle: supervise peer при P1 | start/stop/status другого процесу `peer listen --recv`; PID/lock; тести | invite schema IO; QR; GUI onboarding |
| `#83` | E1.1.3 | Invite file export/import | export JSON; import → trust+book; CLI/shared lib | QR raster; GUI |
| `#84` | E1.1.4 | QR PNG encode/decode | той самий PeerInvite ↔ PNG; тести roundtrip | camera; GUI polish |
| `#85` | E1.1.5 | GUI: P1 + invite UX | profile toggle; peer status; export/import/QR show+load file | P2+; camera; інші ОС |

```text
#80 PeerInvite schema
  → #81 settings P1 + peer_listen
    → #82 peer lifecycle supervise
      → #83 invite file export/import
        → #84 QR PNG
          → #85 GUI wiring
```

**RFC:** RFC-S invite (`#80`) → RFC-E settings/lifecycle (`#81`/`#82`) → RFC-E invite+QR (`#83`/`#84` → RFC-0032/RFC-0033) → docs GUI (`#85`).

**Acceptance E1.1:** два Linux Dev Preview інстанси (або loopback dual-root): увімкнули P1 → обмінялись invite файл/QR → trust+book → `peer listen --recv` працює → dial/recv smoke. Без P2–P6.

### 4b. Addendum E2 — macOS — **DONE** (2026-08-22)

**Scope:** паритет поточного Desktop (P0/P1 + GUI + settings з E1+E1.1) як macOS Developer Preview. Той самий UI / `aira-desktop-runtime`.

**Рішення пакування:** `.app` directory + `.tar.gz` (аналог Linux tarball; **не** вимагає `hdiutil`). Codesign / notarize / DMG — **Out** першого E2.

**Атоми → QUEUE `#86`–`#89`**

| # | ID | Scope | Done when | Не в цьому рядку |
|---|----|-------|-----------|------------------|
| `#86` | E2.0 | macOS DesktopPaths | `Application Support` / `Preferences` / `Logs`; тести layout | LaunchAgent; `.app`; DMG |
| `#87` | E2.1 | macOS LaunchAgent autostart | plist write/remove за `autostart_on_login`; Linux XDG лишається | `.app` bundle; notarize |
| `#88` | E2.2 | macOS `.app` + tarball | `scripts/package-desktop-macos.sh` + Info.plist; bins у `Contents/MacOS` | DMG; notarize; Windows |
| `#89` | E2.3 | Docs + RFC macOS | `docs/desktop-packaging-macos.md`; install без `cargo` notes | E3 Windows; App Store |

```text
#86 macOS paths
  → #87 LaunchAgent
    → #88 .app tarball
      → #89 docs
```

**Acceptance E2:** на macOS (або cross-check layout на Linux CI): paths коректні → autostart plist → розпакував `.app` tarball → GUI start P0; P1 optional. Без notarize.

### 4c. Addendum E3 — Windows — **DONE** (2026-08-22)

**Scope:** паритет поточного Desktop (P0/P1 + GUI + settings з E1+E1.1) як Windows Developer Preview. Той самий UI / `aira-desktop-runtime`.

**Рішення пакування:** versioned **`.zip`** + user install script (аналог Linux tarball / macOS `.tar.gz`; **не** вимагає MSI/NSIS на першому E3). Codesign / SmartScreen / Store — **Out** першого E3.

**Атоми → QUEUE `#90`–`#93`**

| # | ID | Scope | Done when | Не в цьому рядку |
|---|----|-------|-----------|------------------|
| `#90` | E3.0 | Windows DesktopPaths | `%LOCALAPPDATA%` / `%APPDATA%`; тести layout | autostart; zip; MSI |
| `#91` | E3.1 | Windows login autostart | Startup shortcut або Registry Run за `autostart_on_login`; Linux/macOS лишаються | zip package; codesign |
| `#92` | E3.2 | Windows zip package | `scripts/package-desktop-windows.sh` + `deploy/windows/` install | MSI; codesign; macOS |
| `#93` | E3.3 | Docs + RFC Windows | `docs/desktop-packaging-windows.md`; install без `cargo` notes | App Store; MSI |

```text
#90 Windows paths
  → #91 login autostart
    → #92 zip package
      → #93 docs
```

**Acceptance E3:** на Windows (або cross-check layout на Linux CI): paths коректні → autostart hook → розпакував zip → GUI start P0; P1 optional. Без codesign.

### 4d. Addendum E4 — network profiles P2–P6 — **DONE** (2026-08-22)

**Рішення розробника:** 2026-08-22 — відкрити **E4** після DONE E3 (`#93`). Лінійно P2 → P3 → P4 → P5 → P6; один OPEN = один Analyze-цикл. Stabilization атоми (branch protection, Handle opacity, verify-on-read, Policy Gate у dispatch, acquisition fail-closed, toolchain pin, `SECURITY.md` refresh, split `model-acquisition`) — **поза E4**, за окремим рішенням.

**Scope:** розширити Desktop runtime + GUI з поточного P0/P1 (E1.1) до профілів P2–P6 з [`desktop-ux.md`](desktop-ux.md) §3. CLI primitives вже є в post-MVP (`docs/peer-link.md`, Phase B `#18`–`#37`); E4 = **settings + supervise + UX**, не новий peer protocol.

**Інваріанти E4 (додатково до §2 та §4a)**

1. Профілі відкриваються **послідовно**: runtime приймає лише профілі ≤ поточного DONE-рівня; вищі — fail-closed до свого settings-атома.
2. HTTP loopback (`aira-node --http`) — **завжди**; peer supervise лише при P1+ (як E1.1).
3. **P2** = P1 + supervised `peer listen --recv --dht --apply-book` (opt-in DHT→address book, `#22`). Без relay / gossip / federation / STUN default.
4. **P3** = P2 peer flags + `--relay` + `relay_ttl_days` у settings (default **31**). Dial `via` — існуюча peer-семантика; Desktop не auto-trust relay strangers.
5. **P4** = P2 peer base + `--gossip`. **Mutex P3|P4:** одночасно `--relay` і `--gossip` на одному `peer listen` — fail-closed у settings normalize.
6. **P5** = окремий wizard `federation join` (descriptor file); **не** змішувати з peer listen flags. Membership status read-only у GUI; Book II federation — Out.
7. **P6** = **Dev / Advanced only** (opt-in panel): STUN query, discv listen, iterative FIND — operator shortcuts над існуючим CLI; **без** публічного STUN default, **без** auto-trust з discovery.
8. Заборонено в Desktop (без змін): `--allow-public-bind`, публічний STUN default, авто-trust невідомих, прихований auto-increment порту, non-loopback `peer_listen` без explicit peer CLI (як E1.1).
9. Settings `$id` лишається `0.1`; нові поля — optional з defaults у runtime (`relay_ttl_days` тощо).
10. C1 / `aira-core` / Book 0 — **не змінювати**. GUI tech — Rust-only egui (без змін).

**Атоми → QUEUE `#94`–`#106`**

| ID | Підфаза | Атом | Done when | Не в цьому рядку |
|----|---------|------|-----------|------------------|
| `#94` | E4.0 | Settings P2 | runtime приймає P2; `peer_listen` як P1; P3+ fail-closed; fixtures/docs | peer `--dht`; GUI |
| `#95` | E4.1 | Lifecycle P2 peer | supervise `peer listen --recv --dht --apply-book`; PID/lock; dual-root DHT→book smoke | P3 relay; GUI |
| `#96` | E4.2 | GUI P2 | profile selector P0\|P1\|P2; peer status (dht mode); invite UX лишається | P3+; camera |
| `#97` | E4.3 | Settings P3 + relay TTL | runtime P3; optional `relay_ttl_days` (default 31); P4+ fail-closed; mutex P3\|P4 зафіксовано в normalize | peer `--relay`; GUI |
| `#98` | E4.4 | Lifecycle P3 relay | supervise `--relay --relay-ttl-days N`; registry reload smoke | gossip; GUI |
| `#99` | E4.5 | GUI P3 Advanced | Advanced relay toggle/status; mutex hint P3 vs P4 | P4 gossip UI; P5 |
| `#100` | E4.6 | Settings P4 gossip | runtime P4; enforce P3\|P4 mutex; P5+ fail-closed | peer `--gossip`; GUI |
| `#101` | E4.7 | Lifecycle P4 gossip | supervise `--gossip` (без `--relay`); forward-filter smoke | relay; GUI |
| `#102` | E4.8 | GUI P4 Advanced | gossip toggle/status у Advanced | P5 wizard; P6 |
| `#103` | E4.9 | Federation join desktop | shared lib/CLI: descriptor file → `federation join`; membership read API; tests | GUI wizard; Book II |
| `#104` | E4.10 | GUI P5 federation | wizard import descriptor; membership status display | P6; remote federation |
| `#105` | E4.11 | P6 Advanced discovery | opt-in Dev panel: STUN/discv/FIND shortcuts (no public STUN default); operator docs hook | auto-trust; camera |
| `#106` | E4.12 | Docs + RFC E4 | `docs/desktop-network-profiles.md` + RFC-0043; cross-links peer-link/desktop-peer/desktop-ux | зміна peer protocol |

```text
#94 settings P2
  → #95 lifecycle P2 (--dht --apply-book)
    → #96 GUI P2
      → #97 settings P3 + relay_ttl
        → #98 lifecycle P3 (--relay)
          → #99 GUI P3
            → #100 settings P4
              → #101 lifecycle P4 (--gossip)
                → #102 GUI P4
                  → #103 federation join lib
                    → #104 GUI P5
                      → #105 P6 Advanced discovery
                        → #106 docs + RFC
```

**RFC:** RFC-E settings/lifecycle per profile (`#94`/`#95`, `#97`/`#98`, `#100`/`#101`) → RFC-E federation desktop (`#103`) → RFC-E discovery advanced (`#105`) → RFC-0043 docs (`#106`).

**Acceptance E4:** на Linux Dev Preview (dual-root smoke достатньо на CI): P2 DHT announce→find→book→dial; P3 relay register/deliver; P4 gossip forward filter; P5 federation join pin; P6 Advanced panel викликає STUN/discv/FIND без public defaults. macOS/Windows — паритет через той самий runtime/GUI (без нових packaging атомів). Позиціонування: **Developer Preview**, не production distributed AIRA.

### Пізніше (не нумерувати зараз)

Stabilization атоми (branch protection, Handle opacity, verify-on-read, Policy Gate у dispatch, acquisition fail-closed, toolchain pin, `SECURITY.md` refresh, split `model-acquisition`) — **виконано** у Phase F [`phase-f-plan.md`](phase-f-plan.md) `#107`–`#119` (**DONE**, 2026-08-24). Далі — Phase G [`phase-g-plan.md`](phase-g-plan.md) `#120`–`#146`.

## 5. RFC / артефакти

- RFC-S: desktop settings schema (`#75`) — [`AIRA-RFC-0024`](../specs/rfc/AIRA-RFC-0024-desktop-settings-payload-schema.md)
- RFC-E: `aira desktop …` + shared lifecycle (`#76`) — [`AIRA-RFC-0025`](../specs/rfc/AIRA-RFC-0025-desktop-lifecycle-cli.md)
- Packaging: [`desktop-packaging.md`](desktop-packaging.md) + RFC-0028 (`#79`); tarball+`.desktop` (не AppImage)
- macOS packaging: [`desktop-packaging-macos.md`](desktop-packaging-macos.md) + RFC-0038 (`#89`)
- Windows packaging: [`desktop-packaging-windows.md`](desktop-packaging-windows.md) + RFC-0042 (`#93`)
- Linux launcher: [`desktop-launcher.md`](desktop-launcher.md) + RFC-0026 (`#77`)
- Desktop GUI + autostart: [`desktop-gui.md`](desktop-gui.md) + RFC-0027 (`#78`)
- Addendum E2 (macOS): §4b → QUEUE `#86`–`#89`; RFC-0035 paths (`#86`); RFC-0036 LaunchAgent (`#87`); RFC-0037 `.app` tarball (`#88`); RFC-0038 docs (`#89`)
- Addendum E3 (Windows): §4c → QUEUE `#90`–`#93`; RFC-0039 paths (`#90`); RFC-0040 autostart (`#91`); RFC-0041 zip (`#92`); RFC-0042 docs (`#93`)
- Addendum E4 (P2–P6): §4d → QUEUE `#94`–`#106` **DONE**; [`desktop-network-profiles.md`](desktop-network-profiles.md) + RFC-0043 (`#106`); per-profile RFC-E у відповідних атомах

## 6. Acceptance E1 (Linux)

Кінцевий користувач на Linux: встановив пакет → клікнув іконку → node на **зафіксованому** loopback endpoint (або attach) → native UI за setting → Stop/Quit → autostart лише якщо увімкнув у Settings (hooks з `#78`). Без peer, без `cargo run`. Позиціонування: **Developer Preview**, не production distributed runtime.

## 7. Acceptance E1.1 (після `#85`)

Див. §4a Acceptance E1.1.