# UltraQA — Analyze-55

## Goal
mTLS client CN must map to TrustStore AiraRef; unknown/revoked fail-closed.

## Scenario matrix

| ID | Intent | Result | Evidence |
|----|--------|--------|----------|
| U1 | Trusted CN | accept | `mtls_accepts_trusted_cn` |
| U2 | Unknown CN (CA ok) | reject | `mtls_rejects_unknown_truststore_cn` |
| U3 | Revoked CN | reject | `mtls_rejects_revoked_truststore_cn` |
| U4 | Wrong CA / anon | reject | existing tests |
| U5 | Full suite + clippy | green | 27 passed; clippy -D warnings |

## Verdict
**PASS**
