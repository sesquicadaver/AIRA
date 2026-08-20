# AIRA-RFC-0033 — Desktop PeerInvite QR PNG

## 1. Summary

Encode/decode the same `aira:schema:desktop:peer-invite:0.1` document as a QR PNG file (`invite-export-qr` / `invite-import-qr`). Compact JSON payload; camera/live scan Out.

## 5. Non-Goals

Camera / live scan (`#85` may show PNG only); DHT/relay; auto-trust.

## 7. Change

- `aira-desktop-runtime::invite_qr`
- CLI DesktopCommands InviteExportQr / InviteImportQr
- Docs: `docs/desktop-invite.md`

## 15. Tests

`cargo test -p aira-desktop-runtime --test invite_qr`
