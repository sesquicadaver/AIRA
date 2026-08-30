# AIRA-RFC-0095 — MSRV aligned with CI + SHA-pinned Actions + cargo-deny

## 1. Summary

Phase I `#197`: workspace `rust-version` matches the CI 1.94 pin; GitHub Actions `uses:` are commit-SHA pinned; `cargo deny check` runs as an informational job (not a merge gate).

## 5. Non-Goals

Phase I close / RFC-0078 (`#198`); making `cargo-deny` a required status check; a 1.75 MSRV job (the 1.75 claim was stale).

## 10. Contract

```text
Cargo.toml rust-version = "1.94"  ↔  rust-toolchain.toml 1.94.0  ↔  CI toolchain: 1.94.0
ci.yml uses: owner/repo@<40-hex>  # tag in comment
job cargo-deny: cargo deny check (deny.toml); not in branch-protection required checks
required checks unchanged: fmt-clippy-test-schema-c0-c1, conformance-c2
deny.toml first-pass: unmaintained = "none" (rustls-pemfile still a direct TLS dep); ignore RUSTSEC-2026-0253 (lru via rqrr)
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test toolchain_pin
cargo test -p aira-desktop-runtime --test ci_governance_doc cargo_deny
cargo test -p aira-desktop-runtime --test phase_i_doc phase_i_msrv_supply_chain_197
```
