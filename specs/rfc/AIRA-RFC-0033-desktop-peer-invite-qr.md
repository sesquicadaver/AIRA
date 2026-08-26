# AIRA-RFC-0033 — Desktop PeerInvite QR PNG

## 1. Summary

Encode/decode the same `aira:schema:desktop:peer-invite:0.1` document as a QR PNG file (`invite-export-qr` / `invite-import-qr`). Compact JSON payload. GUI camera scan (`#133`) decodes in-memory luma frames via `import_invite_qr_luma`.

## 5. Non-Goals

DHT/relay; auto-trust from discovery; non-loopback dial in this slice.

## 7. Change

- `aira-desktop-runtime::invite_qr`
- CLI DesktopCommands InviteExportQr / InviteImportQr
- Docs: `docs/desktop-invite.md`

## 15. Tests

`cargo test -p aira-desktop-runtime --test invite_qr`
`cargo test -p aira-desktop` (`invite_qr_from_luma_roundtrip_smoke`)
