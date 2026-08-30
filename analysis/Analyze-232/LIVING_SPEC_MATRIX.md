# Living spec — Analyze-232 (QUEUE #197)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| rust-version↔CI | `Cargo.toml` `1.94` | `workspace_rust_version_matches_ci_pin` | **DONE** |
| Pin Actions SHA | `.github/workflows/ci.yml` | `ci_actions_are_sha_pinned` | **DONE** |
| deny/audit non-gate | `deny.toml`; job `cargo-deny` | `cargo_deny_optional_ci_job_is_not_merge_gate` | **DONE** |
| RFC | `AIRA-RFC-0095-msrv-supply-chain-ci.md` | `phase_i_msrv_supply_chain_197` | **DONE** |
| Phase I close | RFC-0078 | — | **OUT** (`#198`) |
