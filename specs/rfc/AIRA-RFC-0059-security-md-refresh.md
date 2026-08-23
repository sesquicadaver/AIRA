# AIRA-RFC-0059 — SECURITY.md refresh

## 1. Summary

Phase F `#110`: replace skeleton-only `SECURITY.md` with posture aligned to reference tree — CSU isolation, Desktop fail-closed rules, local HTTP/peer notes, CI gates, reporting.

## 5. Non-Goals

New security features; changing runtime behavior.

## 15. Tests

`cargo test -p aira-desktop-runtime --test security_md_doc`
