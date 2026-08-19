# Provenance — Trust unload / sync

**Decision:** `sync_trust_verifiers(root)` makes `trust.json` authoritative for peer verifying keys in the process keyring.

**Why:** Additive `register_keyring` left revoked peers verifiable until process restart (Analyze-23 deferred).

**Non-goals:** rotation ceremonies, CRL files, unloading signing keys, multi-process IPC of keyring state.
