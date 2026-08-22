# AIRA-RFC-0050 — Desktop settings P4 gossip

## 1. Summary

Extend settings runtime (`#100`) for `network_profile=P4` (gossip). P5+ fail-closed. P3|P4 mutex via single profile enum; P4 clears `relay_ttl_days`.

## 5. Non-Goals

peer `--gossip` lifecycle (`#101`); GUI (`#102`).

## 15. Tests

`cargo test -p aira-desktop-runtime --test settings_p4`; `schema validate --fixtures`.
