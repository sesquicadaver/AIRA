# Правила Analyze-21

Identity Keyring.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Never remove `aira:identity:local-test` from the default keyring
3. Secret key file remains hex; mode 0600 best-effort on create
4. Document keyring behavior in `docs/crypto.md`
