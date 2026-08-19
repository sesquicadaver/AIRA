# Living Spec Matrix — Analyze-85

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Mechanical TLS split | `tls/mod.rs` re-exports used by `main` | `tls::tests::*` |
| Self-signed + CLI paths | `tls/paths.rs` | `self_signed_loads_into_rustls_config`, `resolve_requires_pair` |
| PEM / CA roots | `tls/pem.rs` | `client_ca_empty_fails_closed` |
| CN→TrustStore verifier | `tls/verifier.rs` | `mtls_*`, `assert_cn_helpers` |
| ServerConfig + serve | `tls/serve.rs` | ALPN + handshake mem tests |
| http/ untouched | `crates/aira-node/src/http/` | `http::tests::*` |
