# Analyze-48 scope — HTTP bearer authn

## In
- Optional shared secret for local HTTP API
- Axum middleware: require `Authorization: Bearer <token>` when configured
- Exempt `GET /health`
- CLI: `--http-token`; env: `AIRA_HTTP_TOKEN`
- Unit/integration oneshot tests

## Out
- mTLS / client certificates
- Per-route RBAC / multi-tenant
- Changing TLS PEM load path
- Public bind default

## Verification
```bash
cargo test -p aira-node
cargo clippy -p aira-node -- -D warnings
```
