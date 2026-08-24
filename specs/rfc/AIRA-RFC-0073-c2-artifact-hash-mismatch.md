# AIRA-RFC-0073 — C2 artifact hash mismatch case

## 1. Summary

Phase G `#123`: conformance case `c2.artifact.hash_mismatch` — `ArtifactProtocolAdapter::publish` returns `INVALID_ARTIFACT` when descriptor `content_hash` does not match payload bytes.

## 5. Non-Goals

New artifact types; wire network AP.

## 15. Tests

`cargo run -p aira-cli -- conformance run --profile C2`
