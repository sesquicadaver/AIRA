# AIRA-RFC-0052 — Desktop GUI P4 Advanced (gossip)

## 1. Summary

GUI Advanced section (`#102`): P4 gossip toggle/status; mutex with P3 relay. `apply_network_profile` accepts P4.

## 5. Non-Goals

P5 federation wizard (`#104`); lifecycle `--gossip` (`#101` done).

## 15. Tests

`cargo test -p aira-desktop actions::tests::p4_profile_persist_peer_listen`
