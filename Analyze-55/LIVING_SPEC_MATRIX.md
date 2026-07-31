# LIVING_SPEC_MATRIX — Analyze-55

| ТЗ | Модуль | Тести |
|----|--------|-------|
| CA + trusted CN | `TrustMappedClientVerifier` | `mtls_accepts_trusted_cn` |
| Unknown CN | same | `mtls_rejects_unknown_truststore_cn` |
| Revoked CN | same | `mtls_rejects_revoked_truststore_cn` |
| Wrong CA / anon | same | existing mtls reject tests |
| Docs | `docs/local-node.md` | manual |
| QUEUE #20 | `QUEUE.md` | DONE |
