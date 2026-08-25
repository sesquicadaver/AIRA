# AIRA-RFC-0067 — Stabilize desktop peer lifecycle integration tests on CI

## 1. Summary

Phase F `#118`: reduce flake in `peer_lifecycle_p2` via `serial_test` group `desktop_peer_integration`, ephemeral port retry on bind conflicts, and settle delay after stop.

Phase G `#131`: same pattern applied to `peer_lifecycle_p3` (relay hub wait loop + serial/retry).

## 5. Non-Goals

P4 lifecycle changes (`#132`); semantic peer protocol changes.

## 15. Tests

`cargo test -p aira-desktop-runtime --test peer_lifecycle_p2`
`cargo test -p aira-desktop-runtime --test peer_lifecycle_p3`
`cargo test -p aira-desktop-runtime --test peer_lifecycle_p2 --test peer_lifecycle --test peer_lifecycle_p3 --test peer_lifecycle_p4 -- --test-threads=8`
