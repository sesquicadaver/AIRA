# Analyze-12 — Epic 5 CSU Runtime

**Scope:** Issue Set #35–#40

## Ralplan (approved)

### Principles
1. CSU is the only Core extension form; in-process Rust trait for MVP
2. Manifest must map to `aira:schema:csu:manifest:0.1`; unsigned rejected
3. Registry checks ABI (`0.1`) + signature presence
4. Lifecycle: Discovered→Registered→Verified→Active→Suspended→Revoked→Archived; invalid transitions rejected; emit events
5. Dispatch only to Active CSU; failures → `CSUFailed`
6. Isolation: context exposes Event/Artifact/Policy APIs only — no Object/Artifact mutation, no peer CSU calls

### Decision
Implement full Epic 5 in `aira-csu` + CLI `aira csu list|register`. Domain CSUs (#41+) stay stubs.

### Acceptance
- Manifest ↔ schema; unsigned fixture rejected
- Register + ABI/signature checks; appears in `aira csu list`
- Lifecycle states + invalid transition reject + transition events
- `Csu` trait: `manifest()` + `on_event()`; outputs Event|Artifact|PolicyQuery|Failure
- Active receives events; Suspended does not; dispatch failure → CSUFailed
- Isolation baseline tests
- cargo test/fmt/clippy PASS; originals untouched

### Out of scope
Epic 6 basic CSU domain logic (#41–#46), WASM/subprocess, crypto verify, operational flow (#47+).
