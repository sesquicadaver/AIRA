# Analyze-99 — DI crystallize (QUEUE #64)

## Interview-complete rationale
QUEUE `#64` + phase-d-plan §6a D4.4 lock Done when / Out.

## Crystallized spec

1. Explicit `activate`: copy `models/verified/…` → `models/cache/…`.
2. Publish ModelInstalled-style Evidence + Event; **no** model execution.
3. Inventory update via CLI orchestration: `scan_and_publish` on `models/cache` (no CSU↛CSU dep).
4. Requires prior verified pointer (`#63`); missing → error.
5. Out: sharing, rating, remote registry, auto-execution, C1/core.
