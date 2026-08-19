# DI crystallize — Analyze-85 / QUEUE #50

## In
1. Replace `tls.rs` with `tls/mod.rs` and four modules: paths, pem, verifier, serve.
2. Keep `main.rs` imports: `resolve_tls_paths`, `serve_https`, `load_client_ca_roots`.
3. Keep `#[cfg(test)]` suite in `tls/mod.rs`.
4. No new TLS modes; CN→TrustStore semantics unchanged.

## Out
`http/` rewrite; health-bind policy; new TLS modes.
