# AIRA-RFC-0039 — Windows DesktopPaths

## 1. Summary

Define Windows system layout for Desktop: `%LOCALAPPDATA%\AIRA` (data + runtime + logs), `%APPDATA%\AIRA\desktop-settings.json` (settings). Linux XDG and macOS layouts unchanged. Testable via `windows_for_profile` / `windows_for_home`.

## 5. Non-Goals

Windows autostart (`#91`); zip packaging (`#92`); MSI/codesign; macOS/Linux changes.

## 7. Change

- `aira-desktop-runtime::paths`
- Docs: phase-e §2.1 / §4c

## 15. Tests

`cargo test -p aira-desktop-runtime paths::`
