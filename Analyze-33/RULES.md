# Правила Analyze-33

Peer CLI ops.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Do not change peer hello/frame wire in this slice
3. `peer add` fail closed if identity not trusted / revoked
4. Default listen remains loopback
5. Document CLI in `docs/peer-link.md`
