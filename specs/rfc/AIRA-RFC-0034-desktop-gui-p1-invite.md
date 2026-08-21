# AIRA-RFC-0034 — Desktop GUI P1 + invite UX

## 1. Summary

Wire E1.1 into `aira-desktop`: P0/P1 profile toggle, peer status, `peer_listen` edit, and PeerInvite file/QR export-import (PNG show + file load; no camera).

## 5. Non-Goals

Camera / live scan; P2+ profiles; macOS/Windows packaging; tray StatusNotifier.

## 7. Change

- `crates/aira-desktop` (`app`, `actions`)
- `encode_invite_rgba` / `encode_invite_luma` for QR preview
- Docs: `docs/desktop-gui.md`

## 15. Tests

`cargo test -p aira-desktop` (smoke P1 + invite roundtrip via `actions`)
