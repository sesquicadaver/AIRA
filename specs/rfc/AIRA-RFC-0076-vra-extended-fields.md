# AIRA-RFC-0076 — VRA extended fields

## 1. Summary

Phase G `#126`: optional extended fields on `aira:schema:result:verified-result-artifact:0.1` (`counter_evidence_refs`, `claim_refs`, `revision_refs`, `epistemic_status`, `contextual_fitness`, `source_output_ref`) plus conformance case `c1.result.extended_fields` and extended fixture.

## 5. Non-Goals

Full runtime payload mapping in verification-basic; Book I 1:1 every field on plane output.

## 15. Tests

`cargo run -p aira-cli -- conformance run --profile C1`; `schema validate --fixtures fixtures`
