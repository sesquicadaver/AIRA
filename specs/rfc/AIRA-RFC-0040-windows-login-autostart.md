# AIRA-RFC-0040 — Windows login autostart

## 1. Summary

When `autostart_on_login=true` on Windows, write `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\AIRA Desktop.bat` launching resolved `aira-desktop.exe`. Disable removes the batch file. Linux XDG and macOS LaunchAgent hooks unchanged.

## 5. Non-Goals

Registry Run keys; `.lnk` COM shortcuts; zip packaging (`#92`); codesign; macOS/Linux changes.

## 7. Change

- `aira-desktop-runtime::autostart` (`windows_startup_bat`, `set_windows_startup_in`, OS dispatch in `set_autostart`)
- Docs: `desktop-gui.md`, `desktop-ux.md`

## 15. Tests

`cargo test -p aira-desktop-runtime --test autostart`
`cargo test -p aira-desktop-runtime --lib autostart`
