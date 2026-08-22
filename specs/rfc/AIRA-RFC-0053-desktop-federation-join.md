# AIRA-RFC-0053 — Desktop federation join library

## 1. Summary

Shared Desktop runtime (`#103`): `join_federation_descriptor_file` + `read_federation_membership`; bootstrap + `aira-protocol::join_federation`. `load_federation_membership` exported from `aira-protocol`.

## 5. Non-Goals

GUI P5 wizard (`#104`); Book II remote federation.

## 15. Tests

`cargo test -p aira-desktop-runtime --test federation_join`
