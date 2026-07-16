# Provenance — Dual-Key Grace Window

**Decision:** Optional `grace_until` (RFC3339 UTC) on CRL entries from rotate; `to_keyring_at` / sync include old pubkey while `now <= until`. CLI `--until` is explicit; omit = immediate cutover.

**Why:** Operator-controlled dual-key window without fixed TTL policy.

**Non-goals:** Auto TTL; unrevoke during grace changing semantics; node secret rotate.
