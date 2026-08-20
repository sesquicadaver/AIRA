# Phase E — Desktop UX & One-Click Start v0.1

**Статус:** складено 2026-08-20; рішення UX — [`desktop-ux.md`](desktop-ux.md).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md). Поза цим планом / addendum поодинокі пункти в QUEUE не додавати.  
**Не канон Book:** не змінює Core / C0–C1 / OperationalPlane semantics.

## 0. Порядок

```text
desktop-ux.md (рішення)
  → цей план
    → лінійний хвіст QUEUE #75+
      → один OPEN = один Analyze-цикл
```

ОС: **Linux → macOS → Windows** по черзі (без паралельної плутанини пакетів).

## 1. Класифікація

```text
Phase E = product shell over existing local plane
  + settings schema
  + lifecycle orchestrator (start/stop/status)
  + Desktop GUI/tray (end-user)
  + Dev edition keeps full CLI
```

**Не** Phase E: новий federation mode, публічний bind default, marketplace, операторський runbook замість Desktop.

## 2. Інваріанти

1. End-user старт = GUI; Developer = GUI опційно + CLI.
2. E1 Linux = **лише P0** (local HTTP). P1+ — Addendum E1.1 (файл/QR), не змішувати з E1.
3. `open_ui_on_start` / `autostart_on_login` — settings; autostart default **off**.
4. HTTP loopback завжди в Desktop; peer не стартує в E1.
5. C1 / `aira-core` / Book 0 — **не змінювати** заради Desktop.
6. Packaging macOS/Windows — лише після DONE Linux E1 (Addendum E2/E3).

## 3. Атоми → QUEUE `#75`–`#79` (перша хвиля E0+E1 Linux P0)

| ID | Підфаза | Атом | Done when | Не в цьому рядку |
|----|---------|------|-----------|------------------|
| `#75` | E0.1 | Desktop settings schema | `aira:schema:desktop:settings:0.1` + fixtures; validate --fixtures | orchestrator; GUI; peer profiles > P0 |
| `#76` | E0.2 | CLI `aira desktop start\|stop\|status` | P0: ensure init+identity; start/stop `aira-node --http` loopback; status = health/pid; читає settings | tray; AppImage; P1; autostart OS hooks |
| `#77` | E1.1 | Linux `.desktop` launcher | встановлюваний `.desktop` → orchestrator start; Stop/Quit через CLI або documented; без peer | tray UI; AppImage; macOS/Win |
| `#78` | E1.2 | Linux tray/GUI (мінімум) | Status + Open UI + Settings (`open_ui_on_start`, `autostart_on_login`) + Quit; старт node через GUI | P1; AppImage; notarize |
| `#79` | E1.3 | Linux packaging | AppImage **або** tarball+`.desktop` + короткі docs; Dev notes (CLI) окремо | macOS DMG; Windows installer; P1 |

```text
#75 settings schema
  → #76 desktop CLI orchestrator (P0)
    → #77 Linux .desktop
      → #78 Linux tray/GUI
        → #79 Linux package
```

## 4. Відкладені addendum (не OPEN, доки E1 не DONE)

### Addendum E1.1 — P1 + онбординг друга (файл/QR)

Після `#79` DONE. Scope: Network profile P1 у Settings; trust+book через файл/QR; `peer listen --recv` loopback. **Не** P2–P6 в тому ж addendum без окремого рішення.

### Addendum E2 — macOS

Паритет E1 (P0 + GUI + settings) як `.app` / DMG; той самий UI-код.

### Addendum E3 — Windows

Паритет E1 як installer + tray.

### Пізніше (не нумерувати зараз)

P2 DHT→book; P3 relay; P4 gossip; P5 federation wizard; P6 discovery — лише після E1.1 і окремого addendum.

## 5. RFC / артефакти

- RFC-S: desktop settings schema (`#75`)
- RFC-E: `aira desktop …` (`#76`)
- Packaging docs під `docs/` + посилання з README після `#79`

## 6. Acceptance E1 (Linux)

Кінцевий користувач на Linux: встановив пакет → клікнув іконку → node на loopback → (за setting) відкрився мінімальний UI → може Stop/Quit → autostart лише якщо увімкнув у Settings. Без peer, без `cargo run`.
