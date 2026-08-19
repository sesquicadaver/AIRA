# Правила Analyze-30

Node signing-secret rotate.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Same `identity_id` only — never invent a second node id
3. Do not put the node id on CRL during rotate
4. Do not revoke / rotate `local-test`
5. Document node vs peer rotate in `docs/crypto.md`
