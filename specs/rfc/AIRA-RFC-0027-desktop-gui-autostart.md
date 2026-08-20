# AIRA-RFC-0027 — Desktop GUI + XDG autostart

## 1. Summary

Add Rust-only `aira-desktop` (eframe/egui) with Status / Settings / Quit using `aira-desktop-runtime`, plus XDG autostart hooks synced from `autostart_on_login`. CLI: `aira desktop gui`.

## 5. Non-Goals

AppImage (`#79`); Tauri/web; P1 peer; StatusNotifier tray icon (window is primary UI this slice).

## 7. Change

- `crates/aira-desktop`
- `aira-desktop-runtime::autostart`
- `deploy/desktop/aira-desktop.desktop`
- Docs: `docs/desktop-gui.md`

## 15. Tests

`cargo test -p aira-desktop-runtime --test autostart`; clippy on `aira-desktop`.
