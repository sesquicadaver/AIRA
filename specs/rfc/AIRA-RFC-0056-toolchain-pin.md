# AIRA-RFC-0056 — Rust toolchain pin

## 1. Summary

Phase F `#107`: pin workspace Rust to **1.94.0** in `rust-toolchain.toml`; GitHub Actions installs via `dtolnay/rust-toolchain` with `toolchain: none` (reads repo file).

## 5. Non-Goals

Nightly channel; per-crate toolchain overrides; `RUSTFLAGS` changes.

## 15. Tests

`cargo test -p aira-desktop-runtime --test toolchain_pin`
