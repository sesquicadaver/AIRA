# AIRA-RFC-0036 — macOS LaunchAgent autostart

## 1. Summary

When `autostart_on_login=true` on macOS, write `~/Library/LaunchAgents/ai.aira.desktop.plist` (`RunAtLoad`) pointing at resolved `aira-desktop`. Disable removes the plist. Linux XDG `.desktop` hooks (`#78`) unchanged.

## 5. Non-Goals

`launchctl bootstrap` / kickstart (login loads agents); `.app` packaging (`#88`); notarize; Windows.

## 7. Change

- `aira-desktop-runtime::autostart` (`launch_agent_plist`, `set_launch_agent_in`, OS dispatch in `set_autostart`)
- Docs: `desktop-gui.md`, `desktop-ux.md`, [`desktop-packaging-macos.md`](../../docs/desktop-packaging-macos.md), [`desktop-packaging-macos.md`](../../docs/desktop-packaging-macos.md), [`desktop-packaging-macos.md`](../../docs/desktop-packaging-macos.md)

## 15. Tests

`cargo test -p aira-desktop-runtime --test autostart`
`cargo test -p aira-desktop-runtime --lib autostart`
