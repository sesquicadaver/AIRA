# Analyze-81 — modularize aira-cli (QUEUE #46)

## Status
CLOSED (QUEUE #46 DONE @ 1704991 / PR #9).

## Done when
`crates/aira-cli/src/main.rs` is a thin entrypoint; handlers live in `commands/*`. CLI clap tests stay green; no new subcommands.

## Out
`crypto.rs` / `tenant.rs` / `http.rs` / `tls.rs` splits; new CLI commands.
