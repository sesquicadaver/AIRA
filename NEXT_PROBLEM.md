# NEXT_PROBLEM — one-click Desktop start

**Status:** RESOLVED / provenance  
**Resolved by:** [`docs/desktop-ux.md`](docs/desktop-ux.md) + [`docs/phase-e-plan.md`](docs/phase-e-plan.md) (`#75`–`#106` **DONE**, 2026-08-22)  
**Post-resolution stabilization:** [`docs/phase-f-plan.md`](docs/phase-f-plan.md) (`#107`–`#119` **DONE**, 2026-08-24)  
**Active backlog:** [`QUEUE.md`](QUEUE.md) Phase L `#217`–`#223` (перший OPEN = `#219`) — [`docs/phase-l-plan.md`](docs/phase-l-plan.md)  
**Не канон:** цей файл не конкурує з `desktop-ux.md` / `phase-e-plan.md` / `QUEUE.md`. Далі — лише історичний problem statement.

## Problem statement (вхід)

Запуск системи має бути максимально спрощений (клік по іконці) для всіх користувачів. Проаналізувати це і знайти варіанти вирішення для Linux, MacOS і Windows.

---

## Діагноз (історичний — до Phase E)

Зараз старт орієнтований на **розробника**, не на користувача:

1. `cargo build` / `cargo run`
2. окремий `--root`
3. `init` → `identity create`
4. окремо `aira-node --http`
5. окремо `peer listen` (якщо потрібен peer)

На Linux є лише **шаблони systemd** (`docs/runbook-systemd.md`) — теж ручні. **Немає** installer’а, desktop-іконки, tray, GUI чи єдиної команди «підняти все».

«Клік по іконці» зараз може означати лише **старт daemon(ів)**. Повноцінний UX (відкрити вікно / сторінку) потребує хоча б мінімального UI поверх HTTP — його в репо немає.

**Стан 2026-08-24:** Desktop Developer Preview реалізовано (lifecycle, GUI, packaging Linux/macOS/Windows, P0–P6 network profiles). Див. [`docs/desktop-gui.md`](docs/desktop-gui.md), [`docs/desktop-packaging.md`](docs/desktop-packaging.md).

---

## Цільова модель (спільна для всіх ОС)

Один артефакт **AIRA** робить:

| Крок | Поведінка |
|------|-----------|
| First run | OS app-data root → `init` + `identity create`, якщо ще немає |
| Start | один lifecycle-оркестратор: HTTP node (P0); peer — profile-dependent |
| Stop | tray Quit / `aira desktop stop` |
| Default | лише loopback; без `--allow-public-bind` |

Ключова зміна в коді: **`aira desktop start|stop|status`** (спільно з GUI через shared lifecycle library).

### Superseded (не реалізовувати як названо тут)

| Було в чернетці | Чинне рішення |
|-----------------|---------------|
| `aira start` / `aira-desktop` | `aira desktop start\|stop\|status` |
| Порядок **A → C → B** (orchestrator → package → GUI) | `#76 → #77 → #78 → #79` (orchestrator → launcher → GUI → package) |
| Optional peer у першому Desktop-релізі | E1 = **лише P0**; P1 = Addendum E1.1 |
| Root `~/.aira` як єдина норма | Desktop = OS app-data / config / runtime / logs (див. phase-e); Dev = `--root` / `.aira` |
| «той самий ярлик» = Start і Stop | launcher → start; stop = CLI або tray Quit |
| auto next port при зайнятості | **fail-closed** або attach до сумісного instance (див. phase-e) |
| Tauri як рівний default | **Rust-only GUI/tray** (egui/native); без Node.js/web build dependency |

---

## Варіанти по ОС (історичний аналіз)

### Linux

| # | Варіант | UX | Плюси | Мінуси |
|---|---------|-----|-------|--------|
| **L1** | `.desktop` + release-бінар у `/usr/local` або AppImage | клік у меню | просто, нативно | треба зібрати/встановити бінар |
| **L2** | AppImage / Flatpak «AIRA» | клік = запуск | портативно | sandbox ускладнює peer/мережу |
| **L3** | user systemd + іконка «Start/Stop» | клік + автостарт | надійно для daemon | гірше для «звичайного» юзера |
| **L4** | Tray (Ayatana/StatusNotifier) | іконка в панелі | status/stop/логи | більше UI-роботи |

**Рекомендація Linux (чинна):** L1 + `aira desktop` → tray/GUI (`#78`) → tarball+`.desktop` (`#79`). systemd — для серверів, не для desktop.

### macOS / Windows

Історичні таблиці M1–M4 / W1–W4 лишаються як аналіз. **Реалізовано:** E2 macOS `#86`–`#89`, E3 Windows `#90`–`#93` (`phase-e-plan.md`).

---

## Порівняння стратегій продукту (історичне)

| Стратегія | Суть | Статус |
|-----------|------|--------|
| **A. CLI-оркестратор** | lifecycle ховає init/identity/node | **DONE** `#76` |
| **B. Desktop shell** | tray/GUI | **DONE** `#78` |
| **C. Пакети ОС** | tarball / `.app` / zip | **DONE** `#79`/`#89`/`#93` |
| **D. Лише docs/scripts** | `start.sh` / `.bat` | відхилено |

---

## Уточнення розробника (прийняті)

1. End-user + Developer; оператор вузла — зайвий.
2. Open UI on start — setting; старт node через GUI.
3. Peer: детальні профілі в `desktop-ux.md`; E1 = P0 only (**C**).
4. Autostart — setting (default off); OS hooks у `#78`.
5. ОС по черзі: Linux → macOS → Windows — **виконано** для Developer Preview.

---

## Що далі

Не розширювати цей файл новими рішеннями.

- Desktop acceptance / profiles — [`docs/phase-e-plan.md`](docs/phase-e-plan.md), [`docs/desktop-network-profiles.md`](docs/desktop-network-profiles.md).
- Post-E stabilization — [`docs/phase-f-plan.md`](docs/phase-f-plan.md) (**DONE**).
- Активна лінійна черга — Phase L [`docs/phase-l-plan.md`](docs/phase-l-plan.md); виконання — перший OPEN `#219` у [`QUEUE.md`](QUEUE.md).
