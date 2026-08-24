# AIRA-RFC-0071 — Living spec C0 smoke

## 1. Summary

Phase G `#121`: integration test `living_spec_smoke.rs` asserts bidirectional sync between C0 case ids in `docs/implementation-status.md` (Conformance profiles table) and `run_c0` test_ids.

## 5. Non-Goals

Full living-spec matrix autogen; C1/C2 sync in this atom.

## 15. Tests

`cargo test -p aira-desktop-runtime --test living_spec_smoke`
