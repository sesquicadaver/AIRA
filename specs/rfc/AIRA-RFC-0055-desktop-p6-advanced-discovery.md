# AIRA-RFC-0055 — Desktop P6 Advanced discovery

## 1. Summary

P6 operator shortcuts (`#105`): explicit STUN Binding query, UDP discv announce, iterative FIND — runtime (`aira-desktop-runtime::discovery`) + GUI Dev panel + `actions` wrappers. No public STUN default; no auto-trust from discovery.

## 5. Non-Goals

Camera QR; automatic trust/book updates from FIND; production distributed discovery UX (`#106` docs consolidation).

## 15. Tests

`cargo test -p aira-desktop-runtime --test discovery_operator`
`cargo test -p aira-desktop actions::tests::p6_discovery_stun_fail_closed`
