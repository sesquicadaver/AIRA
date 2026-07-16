# Правила Analyze-25

Minimal Trust CRL.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Never revoke local-test
3. Revoked ids cannot be re-trusted via upsert/add without separate unrevoke (out of scope)
4. Document CRL in `docs/crypto.md`
