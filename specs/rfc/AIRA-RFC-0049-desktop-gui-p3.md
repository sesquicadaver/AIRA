# AIRA-RFC-0049 — Desktop GUI P3 Advanced (relay)

## 1. Summary

GUI Advanced section (`#99`): P3 relay hub toggle, `relay_ttl_days` edit, peer status labels for relay+TTL, mutex hint P3 vs P4. `apply_network_profile` accepts relay TTL for P3.

## 5. Non-Goals

P4 gossip UI (`#102`); lifecycle `--relay` (`#98` done).

## 15. Tests

`cargo test -p aira-desktop actions::tests::p3_profile_persist_relay_ttl`
