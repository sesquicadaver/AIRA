# Rules — Analyze-40

- Append-only JSONL; never rewrite history
- Never log Ed25519/X25519 secrets
- Fail visibly if audit append fails after ceremony persist
- No Manifesto / Meditation / gossip / multi-tenant keyring
