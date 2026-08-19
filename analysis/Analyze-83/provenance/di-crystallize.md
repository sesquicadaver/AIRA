# DI crystallize — Analyze-83 / QUEUE #48

## In
1. Replace `tenant.rs` with `tenant/mod.rs` and five modules: paths, map, persist, ceremony, prune.
2. Keep public `aira_object` tenant re-exports identical (`lib.rs` unchanged except module path).
3. Keep `#[cfg(test)]` suite in `tenant/mod.rs`.
4. `decode_csu_dir_name` stays crate-private (`pub(super)`); tests import via `paths`.

## Out
`crypto.rs` rewrite; HTTP authz semantics; new CLI/flags.
