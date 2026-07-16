# Analyze-30 — Node Signing-Secret Rotate

**Scope:** Same `identity_id`, rewrite `local.ed25519` + descriptor; trust upsert (no CRL).

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. Node rotate ≠ peer `trust rotate` (peer changes identity_id + CRL)
2. Keep `aira:identity:{name}`; only key material changes
3. Immediate cutover — no dual-key grace for same `key_ref` in this slice
4. Fail closed on missing identity or pubkey/secret mismatch
5. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `rotate_node_signing_secret(root, new_signing)` + CLI `identity rotate`; trust via `ensure_trust_defaults` / upsert
- **Why:** Fixed file layout + stable `primary_signer` / history `key_ref`
- **Reject:** New identity_id; CRL grace on same id (Keyring single pubkey per ref)
- **Follow-ups:** dual-key same-ref; secret backup file; audit log

### Acceptance
- After rotate: sign/verify with new key OK; `primary_signer` unchanged
- Old signatures under same `key_ref` fail verify
- `trust list` shows same id, new pubkey; no CRL entry for node
- Missing identity → error
- `cargo test -p aira-object` + workspace clippy PASS
- `docs/crypto.md` documents node vs peer rotate
