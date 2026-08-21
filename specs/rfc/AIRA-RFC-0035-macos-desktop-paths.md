# AIRA-RFC-0035 — macOS DesktopPaths

## 1. Summary

Define macOS system layout for Desktop: Application Support (data + runtime), Preferences (settings JSON), Logs. Linux XDG layout unchanged. Testable via `macos_for_home` / `linux_xdg_for_home`.

## 5. Non-Goals

LaunchAgent (`#87`); `.app` packaging (`#88`); DMG/notarize; Windows.

## 7. Change

- `aira-desktop-runtime::paths`
- Docs: phase-e §2.1 / §4b

## 15. Tests

`cargo test -p aira-desktop-runtime paths::`
