# Analyze-81 — modularize aira-cli (QUEUE #46)

## Status
OPEN (implementation on branch `analyze-81-modularize-aira-cli`).

## Done when
`crates/aira-cli/src/main.rs` is a thin entrypoint; handlers live in `commands/*`. CLI clap tests stay green; no new subcommands.

## Out
`crypto.rs` / `tenant.rs` / `http.rs` / `tls.rs` splits; new CLI commands.
