# ULTRAQA — Analyze-79

**Verdict:** PASS (local workspace; filled after cargo)  
**Date:** 2026-08-19

## Goal
No runtime envelope/identity verify fallback to test-domain message.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | envelope | domain-signed envelope | `validate_signature` err | `envelope_rejects_local_test_domain_fallback` | PASS |
| U2 | envelope | payload_hash-signed | ok | same test + EP/AP publish | PASS |
| U3 | identity | domain sig vs identity_id | verify err | `identity_rejects_local_test_domain_signature` | PASS |
| U4 | schema | fixture envelope | schema ok; crypto via re-sign | `envelope_schema_valid_and_unsigned_rejected` | PASS |
| U5 | gate | workspace test + clippy `-D warnings` | green | see cargo | PASS |
