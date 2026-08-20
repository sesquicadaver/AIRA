# AIRA Desktop GUI (QUEUE #78)

**Binary:** `aira-desktop`  
**CLI:** `aira desktop gui`  
**Tech:** Rust-only (`eframe`/`egui`); no Node.js/Tauri.

## Features

| Control | Behavior |
|---------|----------|
| Status | lifecycle label + pid/listen/instance |
| Start / Stop / Refresh | shared `aira-desktop-runtime` |
| Settings | `open_ui_on_start`, `autostart_on_login` (persisted) |
| Quit | stop node + close window |
| Autostart | XDG `~/.config/autostart/aira-desktop.desktop` when enabled |

System tray StatusNotifier icon is **not** required for this slice: the native window is the primary UI. Autostart launches `aira-desktop`.

## Run

```bash
cargo build -p aira-desktop -p aira-node -p aira-cli
./target/debug/aira-desktop --data-root /tmp/aira-gui --node-bin ./target/debug/aira-node
# or:
./target/debug/aira desktop gui --data-root /tmp/aira-gui --node-bin ./target/debug/aira-node
```

Menu entry (GUI): [`deploy/desktop/aira-desktop.desktop`](../deploy/desktop/aira-desktop.desktop) — via `aira desktop launcher-install` or the Linux tarball ([`desktop-packaging.md`](desktop-packaging.md)).

Headless start without UI: set `open_ui_on_start=false` in settings, or use `aira desktop start`.

## Related

- Packaging: [`desktop-packaging.md`](desktop-packaging.md)
- Launcher: [`desktop-launcher.md`](desktop-launcher.md)
- UX canon: [`desktop-ux.md`](desktop-ux.md)
- Plan: [`phase-e-plan.md`](phase-e-plan.md)
