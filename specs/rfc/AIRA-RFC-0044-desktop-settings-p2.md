# AIRA-RFC-0044 — Desktop settings P2 (DHT profile prep)

## 1. Summary

Extend Desktop settings runtime (`#94`) so `network_profile=P2` is accepted with the same `peer_listen` validation as P1 (default `127.0.0.1:9797`). P3+ remain fail-closed. Schema `$id` stays `0.1`. Peer supervise `--dht --apply-book` is `#95`.

## 5. Non-Goals

peer `--dht` lifecycle (`#95`); GUI profile picker (`#96`); P3–P6.

## 7. Change

- `aira-desktop-runtime::settings` — `is_supported` through P2; `requires_peer_listen` for P1|P2
- `apply_network_profile` accepts P2 (shared listen rules)
- Fixture `fixtures/valid/desktop/settings-p2.json`
- Schema field description for P2 acceptance

## 15. Tests

`cargo test -p aira-desktop-runtime --test settings_p2`; unit normalize; `schema validate --fixtures`.
