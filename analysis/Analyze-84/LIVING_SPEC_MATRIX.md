# Living Spec Matrix — Analyze-84

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Mechanical HTTP split | `http/mod.rs` router + re-export | `http::tests::*` |
| AppState | `http/state.rs` | health / capabilities / bearer |
| Bearer gate | `http/auth.rs` | `http_bearer_*`, `bearer_credential_parses_case_insensitive` |
| Helpers | `http/util.rs` | artifact hex / path ids via handlers |
| Handlers | `http/handlers.rs` | problems / CSU / conformance / tenant 403 |
| tls.rs untouched | `crates/aira-node/src/tls.rs` | `tls::tests::*` |
| tenant_auth untouched | `crates/aira-node/src/tenant_auth.rs` | tenant_register / tenant_list |
