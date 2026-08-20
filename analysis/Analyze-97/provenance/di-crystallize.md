# Analyze-97 — DI crystallize (QUEUE #62)

## Interview-complete rationale
QUEUE `#62` + phase-d-plan §6a D4.2 lock Done when / Out. Ambiguity below threshold.

## Crystallized spec

1. After policy **ALLOW**, local `--source` file is copied into `<root>/models/quarantine/…`.
2. DENY → no copy (exit 2); remote URL schemes rejected.
3. Emit Event (`op:quarantine-fetched:…`) + pointer; **no** hash/signature verify (`#63`), **no** activate (`#64`).
4. CLI: `aira models download --model-ref … --source <path>`; without `--source` keep gate-only (`#61`).
5. CSU `network=none`; FS scoped under `models` for write destination; source may be any local path.
6. Out: HTTP, verify, activate, inventory promote, C1/core.
