# AIRA-RFC-0075 — Claim/Assumption artifact fixtures

## 1. Summary

Phase G `#125`: JSON Schema `aira:schema:evidence:claim-artifact:0.1` encodes Book 0 §6.2 claim coordinates and Conformance B0-005 — a `claim_kind` of `Claim` requires at least one `evidence_refs` entry; `Assumption` and `Hypothesis` may omit evidence.

## 5. Non-Goals

Epistemic CSU runtime; assessment roundtrip (#141).

## 15. Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures`
