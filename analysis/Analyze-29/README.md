# Analyze-29 — Per-CSU Publisher Identity

**Scope:** CSU emits sign as `manifest.publisher_identity`; default remains node primary.

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. `identity_ref` = CSU logical identity (node/primary by default)
2. `publisher_identity` = who signs CSU emits (may differ)
3. Missing publisher signing key → hard fail (no local-test fallback)
4. Plane ProblemStatement / lifecycle stay on primary_signer
5. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `signature_for` + `make_event_as` / `make_artifact_as`; basic CSUs emit via `publisher_identity`; optional `with_publisher`
- **Why:** schema fields already exist; avoids mutating global primary during dispatch
- **Alternatives:** swap primary_signer per CSU (racy); only document override (no code)
- **Follow-ups:** node signing-secret rotate; runtime/lifecycle publisher; multi-tenant keyring

### Acceptance
- Default path unchanged (publisher == primary)
- Override: CSU emit producer + key_ref == publisher; verify OK
- NoSigningKey when publisher lacks signing material
- workspace tests + clippy PASS
