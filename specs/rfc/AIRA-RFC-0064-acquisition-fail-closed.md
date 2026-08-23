# AIRA-RFC-0064 — Acquisition fail-closed audit

## 1. Summary

Phase F `#115`: regression audit that model acquisition download/publish paths remain fail-closed without explicit policy ALLOW; decision evidence + docs.

## 5. Non-Goals

Remote URL download; split `model-acquisition` (#116).

## 15. Tests

`cargo test -p aira-csu-model-acquisition`
`cargo test -p aira-conformance --lib`
`docs/model-acquisition-policy.md`
