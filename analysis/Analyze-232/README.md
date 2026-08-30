# Analyze-232 — MSRV + supply-chain CI (QUEUE #197)

## Done
- `rust-version = "1.94"` matches `rust-toolchain.toml` / CI 1.94.0
- `ci.yml` `uses:` SHA-pinned; `deny.toml` (`unmaintained = "none"`) + informational `cargo-deny` job
- RFC-0095; QUEUE `#197` **DONE**; first OPEN `#198`

## Out
Phase I consolidating RFC-0078 (`#198`); making cargo-deny a merge gate.
