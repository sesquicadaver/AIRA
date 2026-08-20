# Analyze-98 — DI crystallize (QUEUE #63)

## Interview-complete rationale
QUEUE `#63` + phase-d-plan §6a D4.3 lock Done when / Out.

## Crystallized spec

1. Verify quarantined weights against ModelArtifact `content_hash` + cryptographic `signature`.
2. Mismatch or unsigned/invalid sig → **reject** + Evidence/Event; file **stays** in quarantine.
3. Match → promote copy to `<root>/models/verified/…` + pointer + Evidence; **no** activate (`#64`).
4. CLI: `aira models verify --artifact <model-artifact.json>` (uses `quarantine.latest.json`).
5. Out: activate, inventory promote, HTTP, C1/core.
