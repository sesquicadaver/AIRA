# DI crystallize — Analyze-80 / QUEUE #45

## In
1. Workspace Cargo graph check from `cargo metadata` (no new crate).
2. `aira-core` must not reach `aira-node`, `aira-peer`, or `csu/*` packages (any dep kind, including transitive).
3. Concrete CSU packages must not depend on each other.
4. Directed cycles on normal+build edges are errors.
5. CI runs `--self-test` (synthetic violations fail) then the live graph.

## Out
Split of CLI/crypto/tenant/http/tls; adding workspace members.
