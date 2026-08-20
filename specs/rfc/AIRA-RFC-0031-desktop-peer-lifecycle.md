# AIRA-RFC-0031 — Desktop P1 peer lifecycle

## 1. Summary

When `network_profile=P1`, Desktop lifecycle supervises a second process: `aira --root <data> peer listen --bind <peer_listen> --recv`, with PID/lock under the Desktop runtime dir. HTTP node supervision is unchanged.

## 5. Non-Goals

invite export/import; QR; GUI P1 controls; DHT/relay/gossip; non-loopback peer bind (`--explicit`).

## 7. Change

- `aira-desktop-runtime::peer`
- `start`/`stop`/`status` peer fields
- CLI prints `peer_pid` / `peer_listen`

## 15. Tests

`cargo test -p aira-desktop-runtime --test peer_lifecycle`
