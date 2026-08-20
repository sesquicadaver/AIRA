# Phase E — Desktop UX & One-Click Start v0.1

**Статус:** складено 2026-08-20; acceptance посилено 2026-08-20 (audit fold-in, без нових атомів перед `#75`).  
**Рішення UX:** [`desktop-ux.md`](desktop-ux.md). Provenance проблеми: [`NEXT_PROBLEM.md`](../NEXT_PROBLEM.md) (**RESOLVED**).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md). Поза цим планом / addendum поодинокі пункти в QUEUE не додавати.  
**Не канон Book:** не змінює Core / C0–C1 / OperationalPlane semantics.  
**Product posture:** Linux E1 = **Developer Preview** shell над reference local plane (не production distributed AIRA). Stabilization P0/P1 поза Desktop (Handle, verify-on-read, CI branch protection тощо) — **окремі майбутні атоми після або паралельно за рішенням розробника**, не блокують і не переставляють `#75`.

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
| Node root | OS application-data (напр. Linux `~/.local/share/aira/`) | explicit `--root` (default `.aira` / CLI convention) |
| Settings | OS configuration (напр. `~/.config/aira/desktop-settings.json`) | той самий schema; path через flag або colocated |
| PID / lock | OS runtime (напр. `XDG_RUNTIME_DIR/aira/` або `~/.local/state/aira/`) | under root або runtime dir |
| Logs | OS log/cache (bounded) | under root or log dir |

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

## 4. Відкладені addendum (не OPEN, доки E1 не DONE)

### Addendum E1.1 — P1 + онбординг друга (файл/QR)

Після `#79` DONE. Scope: Network profile P1 у Settings; trust+book через файл/QR; `peer listen --recv` loopback. **Не** P2–P6 в тому ж addendum без окремого рішення.

### Addendum E2 — macOS

Паритет E1 (P0 + GUI + settings) як `.app` / DMG; той самий UI-код / shared lib.

### Addendum E3 — Windows

Паритет E1 як installer + tray.

### Пізніше (не нумерувати зараз)

P2–P6; окремі stabilization атоми (branch protection, Handle opacity, verify-on-read, Policy Gate у dispatch, acquisition fail-closed, toolchain pin, `SECURITY.md` refresh, split `model-acquisition`) — **поза Phase E планом**, за окремим рішенням розробника в QUEUE.

## 5. RFC / артефакти

- RFC-S: desktop settings schema (`#75`) — [`AIRA-RFC-0024`](../specs/rfc/AIRA-RFC-0024-desktop-settings-payload-schema.md)
- RFC-E: `aira desktop …` + shared lifecycle (`#76`) — [`AIRA-RFC-0025`](../specs/rfc/AIRA-RFC-0025-desktop-lifecycle-cli.md)
- Packaging docs під `docs/` + README після `#79`
- Позначити `NEXT_PROBLEM.md` як RESOLVED (вже)

## 6. Acceptance E1 (Linux)

Кінцевий користувач на Linux: встановив пакет → клікнув іконку → node на **зафіксованому** loopback endpoint (або attach) → native UI за setting → Stop/Quit → autostart лише якщо увімкнув у Settings (hooks з `#78`). Без peer, без `cargo run`. Позиціонування: **Developer Preview**, не production distributed runtime.
