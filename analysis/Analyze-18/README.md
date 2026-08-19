# Analyze-18 — Epic 11 MVP Alpha Release

**Scope:** Issue Set #76–#80

## Ralplan (approved)

### Principles
1. Document demos + developer guides under `docs/`
2. Security baseline: unsigned CSU/artifact rejected; private artifact resolve denied by default; secrets not in event payloads
3. Alpha release pack: binaries build, schemas/fixtures, conformance report, release notes
4. DoD acceptance test covers init → identity → 2+2 → failure evidence → C0/C1

### Acceptance
- #76 `docs/demo.md`
- #77 `docs/csu-development.md`, `local-node.md`, `conformance.md`
- #78 security baseline tests green
- #79 release notes + prepare script + sample conformance report path
- #80 acceptance suite / checklist verified
- cargo test/fmt/clippy PASS; originals untouched

### Out of scope
Real crypto beyond TESTSIG, network federation, GitHub Releases upload automation
