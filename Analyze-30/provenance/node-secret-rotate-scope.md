# Provenance — Node signing-secret rotate

## Decision
Same `identity_id`, rewrite `local.ed25519` + JSON, trust upsert. Not peer `TrustStore::rotate`.

## Why
Fixed node file layout; `primary_signer` continuity; Keyring is one pubkey per `key_ref`.

## Rejected for MVP
- New identity_id (peer-style)
- CRL / dual-key grace on same id
- Durable secret backup file

## Upstream tip
`6db5225` (Analyze-29 lockfile)
