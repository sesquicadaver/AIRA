# AIRA-RFC-0012 — Quarantine verify hash/signature (RFC-D/E)

## 1. Summary

`verify_quarantine` checks the latest quarantined weight file against a signed ModelArtifact (`aira:schema:model:artifact:0.1`): cryptographic signature over the artifact body (without `signature`) and `content_hash` equality with the observed file hash. Mismatch/unsigned → reject Evidence + Event; file stays in quarantine. Match → copy to `<root>/models/verified/…` (no activate).

## 2. Problem Statement

Quarantine staging (`#62`) must not become activation. D4.3 proves integrity before `#64`.

## 3. Scope

- `verify_quarantine(root, artifact_path)`
- CLI: `aira models verify --artifact <path>`
- Pointer `models/verified.latest.json`
- reason_refs: `model-hash-mismatch`, `model-unsigned`, `model-signature-invalid`, `model-verified`

## 4. Non-Goals

```text
activate / inventory promote (#64)
remote HTTP
C1 / aira-core
```

## 5. Semantics

| Condition | Result | Exit |
|-----------|--------|------|
| TESTSIG / missing sig | Rejected (unsigned) | 2 |
| bad ed25519 | Rejected | 2 |
| hash mismatch | Rejected; quarantine kept | 2 |
| hash+sig OK | Verified staging | 0 |

## 6. Rollback

Remove `verify_quarantine` and CLI `verify`.

## 7. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6a D4.3; QUEUE `#63`.
