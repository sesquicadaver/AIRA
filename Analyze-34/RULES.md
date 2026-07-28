# Правила Analyze-34

Peer listen daemon.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Do not change peer hello/frame wire encoding
3. Default listen remains loopback (`listen`, not `listen_explicit`)
4. Trust gate for address book / dial unchanged
5. Document CLI in `docs/peer-link.md`
