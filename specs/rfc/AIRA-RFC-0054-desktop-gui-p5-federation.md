# AIRA-RFC-0054 — Desktop GUI P5 federation

## 1. Summary

GUI federation wizard (`#104`): import descriptor JSON → join pin; membership status display. Actions: `join_federation_descriptor`, `federation_membership`.

## 5. Non-Goals

P6 discovery (`#105`); remote Book II federation.

## 15. Tests

`cargo test -p aira-desktop actions::tests::p5_federation_join_roundtrip`
