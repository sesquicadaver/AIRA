# AIRA-RFC-0051 — Desktop peer lifecycle P4 (gossip)

## 1. Summary

Supervise `peer listen --recv --dht --apply-book --apply-trust --gossip` when `network_profile=P4` (`#101`). No `--relay`. Forward-filter smoke via hostile trust-delta skip.

## 5. Non-Goals

GUI P4 Advanced (`#102`); relay P3.

## 15. Tests

`cargo test -p aira-desktop-runtime --test peer_lifecycle_p4`
