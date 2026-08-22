# AIRA-RFC-0047 — Desktop settings P3 + relay_ttl_days

## 1. Summary

Extend settings runtime (`#97`) for `network_profile=P3` with optional `relay_ttl_days` (default 31). P4+ fail-closed. P3|P4 mutex via single profile enum.

## 5. Non-Goals

peer `--relay` lifecycle (`#98`); GUI (`#99`).

## 15. Tests

`cargo test -p aira-desktop-runtime --test settings_p3`; `schema validate --fixtures`.
