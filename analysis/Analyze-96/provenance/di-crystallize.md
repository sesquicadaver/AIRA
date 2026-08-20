# Analyze-96 — DI crystallize (QUEUE #61)

## Interview-complete rationale
QUEUE `#61` + [`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6a D4.1 already lock inputs/outputs/Out. Ambiguity below quick threshold; no user rounds needed.

## Crystallized spec

1. **ALLOW path:** policy present + `auto_download=true` → `GateDecision::Allow`, reason_ref `aira:reason:auto-download-true`, decision `CustomArtifact` (`decision: ALLOW`) + `CustomEvent` (`op:policy-allowed:download:…`).
2. **DENY `#60` preserved:** no policy → DENY; `auto_download=false` → DENY; same reason_refs as RFC-0009.
3. **No byte transfer:** ALLOW ≠ fetch; no quarantine/weights/CAS remote write in this row.
4. **CLI:** `aira models download` → exit **0** on ALLOW (`status policy-allowed`); exit **2** on DENY (`status policy-denied`).
5. **Out:** `#62` quarantine, `#63` verify, `#64` activate, remote HTTP, sharing, C1/core.

## Decision boundary (agent may decide)
- Artifact id prefix `acq-allow` / event id `acq-allow-…`.
- Reason text wording; stable reason_ref as above.
- Keep `download-not-implemented-d3` **removed** from ALLOW path (superseded by D4.1 ALLOW).
