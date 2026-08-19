# ULTRAQA — Analyze-74

**Verdict:** PASS  
**Date:** 2026-08-19

| ID | Scenario | Expected | Status |
|----|----------|----------|--------|
| U1 | Permuted keys / no spaces | same hash | PASS (unit) |
| U2 | Top-level signature omitted from hash | same hash | PASS |
| U3 | Mutate event_type | verify fails | PASS |
| U4 | Sign over payload_hash string only | helper sig does not verify | PASS |
| U5 | Legacy payload_hash signature | still verifies via `verify_ed25519`; helper rejects | PASS |
