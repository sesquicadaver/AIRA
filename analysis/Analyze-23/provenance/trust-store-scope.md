# Provenance — Trust Store scope

**Decision:** `.aira/identity/trust.json` holds verifying-only Ed25519 public keys; `register_trust_store` merges into the process Keyring used by `verify_ed25519`.

**Why:** Peers can be verified without importing their signing material; extends Alpha.2/21/22 without Core redesign.

**Non-goals this cycle:** rotation ceremonies, CRL/revocation, unloading keys from process ring on remove, SQLite-backed trust.
