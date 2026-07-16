# Analyze-20 — Alpha.2 Real Ed25519

**Scope:** Promote TESTSIG → deterministic Ed25519 sign/verify (post-MVP alpha.2)

## Ralplan (APPROVED — consensus)

### Principles
1. Cryptographic verify replaces empty/TESTSIG presence-only checks on admission paths
2. Deterministic `aira:identity:local-test` keypair (fixed seed) for reproducible tests/fixtures
3. Canonical message = content/payload/csu id domain bytes (documented)
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### Decision Drivers
1. Cross-cycle TODO (Analyze-15…19) and Book II “MUST verify signatures”
2. Smallest vertical slice: local-test key verify before multi-key registry
3. Keep schema `signature_value: string` unchanged

### ADR
- **Decision:** Crypto helpers live in `aira-object::crypto`; stores/manifest/envelope call `verify_ed25519`
- **Why:** Signature type already owned by `aira-object`; single source of truth
- **Alternatives:** New `aira-crypto` crate (deferred); soft-allow TESTSIG forever (rejected)
- **Consequences:** Helpers/`make_*` emit real hex; invalid/TESTSIG rejected on publish/register/append
- **Follow-ups:** Verify against `identity create` key files; multi-key trust store

### Acceptance
- Sign/verify roundtrip tests pass
- Invalid signature rejected on artifact publish + CSU register + event append + protocol envelope
- `cargo test -p aira-object -p aira-artifact -p aira-event -p aira-csu -p aira-protocol -p aira-conformance -p aira-flow -p aira-node` PASS
- clippy `-D warnings` on touched crates; originals untouched

### Out of scope
TLS, federation crypto, hardware keys, non-ed25519 algorithms
