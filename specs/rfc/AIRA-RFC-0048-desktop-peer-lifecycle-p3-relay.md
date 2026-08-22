# AIRA-RFC-0048 — Desktop peer lifecycle P3 (relay)

## 1. Summary

Supervise `peer listen --relay --relay-ttl-days N` when `network_profile=P3` (`#98`). `PeerPidRecord` stores profile + TTL for attach.

## 5. Non-Goals

GUI Advanced P3 (`#99`); gossip P4.

## 15. Tests

`cargo test -p aira-desktop-runtime --test peer_lifecycle_p3`
