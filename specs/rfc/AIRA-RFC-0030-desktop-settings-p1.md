# AIRA-RFC-0030 — Desktop settings P1 + peer_listen

## 1. Summary

Extend Desktop settings runtime (`#81`) so `network_profile=P1` is accepted with validated `peer_listen` (default `127.0.0.1:9797`). P2+ remain fail-closed. Schema `$id` stays `0.1`.

## 5. Non-Goals

peer process supervise (`#82`); invite IO/QR/GUI; P2–P6.

## 7. Change

- `aira-desktop-runtime::settings` normalize/validate
- Fixture `fixtures/valid/desktop/settings-p1.json`
- Schema field descriptions for P1

## 15. Tests

`cargo test -p aira-desktop-runtime --test settings_p1`; unit normalize; `schema validate --fixtures`.
