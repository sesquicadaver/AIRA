# AIRA-RFC-0060 — Handle opacity conformance test

## 1. Summary

Phase F `#111`: C0 case `c0.object.handle_opacity` (B1-003) — Handle `Debug` does not leak storage token or path-like substrings; CSU-visible surface is `object_ref` only.

## 5. Non-Goals

Changing `Handle` API or hiding `storage_token()` from store internals.

## 15. Tests

`cargo test -p aira-conformance`
`cargo run -p aira-cli -- conformance run --profile C0`
