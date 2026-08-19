# DI crystallize — Analyze-81 / QUEUE #46

## In
1. Move clap types to `cli.rs`.
2. Move handlers to `commands/*` matching EVO-2 names (schema, identity, csu, problem, peer, trust, tenant) plus init/status, federation, conformance already present in CLI.
3. Shared helpers in `support.rs`.
4. `main.rs` parses and dispatches only.

## Out
New flags/subcommands; splitting `crypto.rs`/`tenant.rs`/`http.rs`/`tls.rs`.
