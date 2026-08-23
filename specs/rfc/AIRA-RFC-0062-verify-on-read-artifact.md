# AIRA-RFC-0062 — Artifact verify-on-read

## 1. Summary

Phase F `#113`: `CasArtifactStore::resolve` / `resolve_with_access` re-verify canonical artifact descriptor signatures on read, validate CAS bytes against `content_hash`, and verify sidecar JSON when present.

## 5. Non-Goals

New artifact types; policy dispatch (#114).

## 15. Tests

`cargo test -p aira-artifact`
`cargo test -p aira-conformance --lib`
`cargo run -p aira-cli -- conformance run --profile C0` (case `c0.artifact.verify_on_read`)
