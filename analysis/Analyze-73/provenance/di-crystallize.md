# DI crystallize — Analyze-73 / QUEUE #38

## In
1. `.github/workflows/ci.yml` after `cargo test`: `schema validate --fixtures fixtures`, then `conformance run --profile C0`, then `conformance run --profile C1`.
2. Same CLI flags as README. Non-zero process exit fails the job (already implemented in `aira-cli`).
3. `--locked` so CI matches `Cargo.lock`.
4. Reports under `${{ runner.temp }}` so the workspace is not polluted.

## Out
C2; schema/fixture content edits unless the current tree already fails (it does not); crypto; split crates.

## Interview-complete rationale
QUEUE #38 + `docs/phase-c-plan.md` already freeze A vs B vs C. No remaining ambiguity that would change the workflow steps.
