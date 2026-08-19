# DI crystallize — Analyze-86 / QUEUE #51

## In
1. Canonical operator page `docs/operational-plane.md` with EVO-2 §3.3 wording.
2. Crate/module rustdoc on `aira-flow` (`lib.rs`, `plane.rs`, `LocalSession`) and conformance C0/C1.
3. Cross-links from demo / local-node / conformance / crypto / CSU development / README Status.
4. Cargo crate description: reference-local demo plane.
5. Comment on `drain_from`: demo safety bound 256, not a scheduler.

## Out
Rewrite of `drain_from` / submit / inject; new scheduler; `docs/implementation-status.md` (#52); CLI output changes.
