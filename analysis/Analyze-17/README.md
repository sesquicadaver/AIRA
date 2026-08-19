# Analyze-17 — Epic 10 Partial Local C2

**Scope:** Issue Set #71–#75

## Ralplan (approved)

### Principles
1. Protocol Envelope / Response match Schema Pack; invalid signature fixtures rejected
2. AIRA-EP maps local event publish; idempotent duplicates; unsupported version → UNSUPPORTED_VERSION
3. AIRA-AP supports artifact publish/resolve with hash validation
4. AIRA-ID Identity Descriptor is schema-valid with public keys + signature
5. Discovery returns Capability (+ provider CSU), never Node; local registry only

### Acceptance
- #71–#75 covered by `aira-protocol` + schemas + fixtures + tests
- cargo test/fmt/clippy PASS; originals untouched

### Out of scope
Epic 11 docs/security baseline (#76+), federation, network protocols
