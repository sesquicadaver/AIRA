# AIRA-RFC-0032 — Desktop PeerInvite file export/import

## 1. Summary

Shared lib + CLI (`aira desktop invite-export|invite-import`) exchange `aira:schema:desktop:peer-invite:0.1` files: export local identity (+ optional addr); import applies trust upsert then optional address-book upsert.

## 5. Non-Goals

Camera / live scan (see RFC-0033 for file QR); GUI (`#85`); DHT/relay; auto-trust from network.

## 7. Change

- `aira-desktop-runtime::invite`
- CLI DesktopCommands InviteExport / InviteImport
- Docs: `docs/desktop-invite.md`

## 15. Tests

`cargo test -p aira-desktop-runtime --test invite_io`
