# UltraQA — Analyze-51 mTLS

## Scenario matrix

| id | intent | setup | command | expected | actual | cleanup |
|----|--------|-------|---------|----------|--------|---------|
| U1 | no HTTPS + CA | init root | `--http --tls-client-ca missing` | exit 1 require HTTPS | exit 1 | tmp root rm |
| U2 | empty CA PEM | self-signed + empty file | `--http --tls-self-signed --tls-client-ca empty` | exit 1 no certs | exit 1 | tmp root rm |
| U3 | missing CA path | self-signed | `--tls-client-ca /tmp/no-such` | exit 1 not found | exit 1 | ok |
| U4 | handshake accept | unit | `mtls_accepts_valid_client_cert` | ok | ok | n/a |
| U5 | handshake anon reject | unit | `mtls_rejects_missing_client_cert` | err | err | n/a |
| U6 | wrong CA reject | unit | `mtls_rejects_wrong_ca_client_cert` | err | err | n/a |
| U7 | Bearer still 401 | unit | coexistence test | 401 then 200 | ok | n/a |
| U8 | dirty worktree | git status before QA | do not clobber unrelated | clean of QA harness | no harness left | n/a |

## Hostile classes covered
Malformed CA (empty), missing flags path, no-HTTPS misuse, wrong-CA cert, anonymous client.

## Verdict
**PASS** (`qa_verdict.clean: true`, skipped: false)

## Evidence
CLI exit codes 1 for U1–U3; `cargo test -p aira-node` 24 passed.
