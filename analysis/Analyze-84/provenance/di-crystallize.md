# DI crystallize — Analyze-84 / QUEUE #49

## In
1. Replace `http.rs` with `http/mod.rs` and four modules: state, auth, util, handlers.
2. Keep public `router`, `health_router`, `AppState` identical.
3. Keep `#[cfg(test)]` suite in `http/mod.rs`.
4. Bearer/tenant authz call-sites unchanged (`tenant_auth.rs` not rewritten).

## Out
`tls.rs` split; new routes; HTTP authz policy change.
