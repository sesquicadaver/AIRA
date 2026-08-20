# AIRA-RFC-0026 — Linux `.desktop` launcher

## 1. Summary

Ship Freedesktop `deploy/desktop/aira.desktop` and CLI `aira desktop launcher-install|launcher-uninstall` so a menu click runs `aira desktop start` (P0). Stop via Desktop Action or CLI.

## 2–5.

Problem: `#77` needs click/menu start without tray (`#78`) or AppImage (`#79`). Non-goals: tray, autostart hooks, packaging, macOS/Win.

## 7. Proposed Change

- Template validated (`Type=Application`, `Exec=aira desktop start`, Stop action).
- Install to `$XDG_DATA_HOME/applications/aira.desktop`.
- Docs: [`docs/desktop-launcher.md`](../../docs/desktop-launcher.md).

## 15. Conformance

`cargo test -p aira-desktop-runtime --test launcher` must pass.

## 19.

Icon asset deferred to `#79` packaging.
