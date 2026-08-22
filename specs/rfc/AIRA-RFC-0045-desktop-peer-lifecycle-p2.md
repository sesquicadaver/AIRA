# AIRA-RFC-0045 — Desktop peer lifecycle P2 (DHT + apply-book)

## 1. Summary

Supervise `peer listen --recv --dht --apply-book` when `network_profile=P2` (`#95`). P1 keeps `--recv` only. `PeerPidRecord` stores profile for attach semantics.

## 5. Non-Goals

P3 relay; GUI P2 (`#96`); settings changes (`#94` done).

## 7. Change

- `aira-desktop-runtime::peer` — profile-aware spawn/attach
- Integration tests `peer_lifecycle_p2.rs` (start + dual-root DHT→book smoke)

## 15. Tests

`cargo test -p aira-desktop-runtime --test peer_lifecycle_p2`; existing `peer_lifecycle` unchanged.
