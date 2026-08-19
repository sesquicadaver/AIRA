# Правила Analyze-24

Trust keyring unload/sync after remove.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Never unload local-test verifying key
3. Signing identities keep verifying keys even if briefly absent from trust file during sync
4. Document sync behavior in `docs/crypto.md`
