# Ralplan — Analyze-69 / QUEUE #34

## Principles
1. Fail-closed: non-loopback HTTP bind is an operator act, not a warning.
2. One flag covers API `--listen` and `--health-listen`.
3. Do not change peer listen or auth defaults.

## Implementation
1. `aira-node` CLI: `--allow-public-bind` (bool, default false).
2. `assert_bind_allowed(addr, allow_public)` — loopback OK; else require flag.
3. Replace current `--listen` warning-and-bind; replace `--health-listen` hard-loopback with the same helper.
4. If public bind + no TLS: eprintln warning (A1).
5. Unit tests on the helper + health_listen resolver; docs + Living Spec + QUEUE on ship.

## Out
Same as DI crystallize Out.

## Done when
Test refusal without flag; docs path; no federation.
