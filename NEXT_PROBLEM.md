# NEXT_PROBLEM — one-click Desktop start

**Status:** RESOLVED / provenance  
**Resolved by:** [`docs/desktop-ux.md`](docs/desktop-ux.md)  
**Implementation plan:** [`docs/phase-e-plan.md`](docs/phase-e-plan.md)  
**Backlog:** [`QUEUE.md`](QUEUE.md) `#75`–`#79` (перший OPEN = `#75`)  
**Не канон:** цей файл не конкурує з `desktop-ux.md` / `phase-e-plan.md` / `QUEUE.md`. Далі — лише історичний problem statement.

## Problem statement (вхід)

Запуск системи має бути максимально спрощений (клік по іконці) для всіх користувачів. Проаналізувати це і знайти варіанти вирішення для Linux, MacOS і Windows.

---

## Діагноз

Зараз старт орієнтований на **розробника**, не на користувача:

1. `cargo build` / `cargo run`
2. окремий `--root`
3. `init` → `identity create`
4. окремо `aira-node --http`
5. окремо `peer listen` (якщо потрібен peer)

На Linux є лише **шаблони systemd** (`docs/runbook-systemd.md`) — теж ручні. **Немає** installer’а, desktop-іконки, tray, GUI чи єдиної команди «підняти все».

«Клік по іконці» зараз може означати лише **старт daemon(ів)**. Повноцінний UX (відкрити вікно / сторінку) потребує хоча б мінімального UI поверх HTTP — його в репо немає.

---

## Цільова модель (спільна для всіх ОС)

Один артефакт **AIRA** робить:

| Крок | Поведінка |
|------|-----------|
| First run | OS app-data root → `init` + `identity create`, якщо ще немає |
| Start | один lifecycle-оркестратор: HTTP node (P0); peer — не в E1 |
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

**Рекомендація Linux (чинна):** L1 + `aira desktop` → tray (`#78`) → AppImage/tarball (`#79`). systemd — для серверів, не для desktop.

### macOS / Windows

Історичні таблиці M1–M4 / W1–W4 лишаються як аналіз. Чинний порядок: **E2 macOS**, **E3 Windows** після DONE Linux E1 (`phase-e-plan.md`).

---

## Порівняння стратегій продукту (історичне)

| Стратегія | Суть | Статус |
|-----------|------|--------|
| **A. CLI-оркестратор** | lifecycle ховає init/identity/node | **канон `#76`** (не `aira start`) |
| **B. Desktop shell** | tray/GUI | **канон `#78`** (після launcher) |
| **C. Пакети ОС** | AppImage / `.app` / MSIX | **канон `#79`** (після GUI) |
| **D. Лише docs/scripts** | `start.sh` / `.bat` | відхилено |

Оптимальний порядок **не** A→C→B, а **A → launcher → B → C** (`#76`→`#79`).

---

## Уточнення розробника (прийняті)

1. End-user + Developer; оператор вузла — зайвий.
2. Open UI on start — setting; старт node через GUI.
3. Peer: детальні профілі в `desktop-ux.md`; E1 = P0 only (**C**).
4. Autostart — setting (default off); OS hooks у `#78`.
5. ОС по черзі: Linux → macOS → Windows.

---

## Що далі

Не розширювати цей файл новими рішеннями. Acceptance lifecycle / port / local HTTP auth / layout — у [`docs/phase-e-plan.md`](docs/phase-e-plan.md) §2–§3. Виконання — перший OPEN `#75` у `QUEUE.md`.
