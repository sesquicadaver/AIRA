# Changelog

## 0.1.5 — 2026-07-10

### Added

- Basic CSU set: context / reduction / execution / verification / evidence / artifact
- `aira_csu::support` helpers; context resolve + supersede APIs

## 0.1.4 — 2026-07-10

### Added

- CSU Manifest / Registry / lifecycle (`aira-csu`)
- In-process `Csu` trait, event dispatch, isolation baseline
- CLI: `aira csu list|register`

## 0.1.3 — 2026-07-10

### Added

- Content-addressed `CasArtifactStore` + `ArtifactDescriptor` (Epic 4)
- Append-only `MemoryEventLog` with `EventSink` / subscriptions
- `PolicyGate` ALLOW|DENY|REQUIRE + `PolicyEvaluated` events
- `InvariantChecker` emitting `InvariantViolation` events

## 0.1.2 — 2026-07-10

### Added

- C0 core types: `AiraRef`, `ContentHash`, `Signature`, opaque `Handle`
- `ObjectDescriptor` with schema validation and forbidden-type rejection
- Immutable `MemoryObjectStore` + `SqliteObjectStore`

## 0.1.1 — 2026-07-10

### Added

- Canonical terminology guardrail (`docs/canonical-terminology.md`)
- Schema Pack JSON schemas under `schemas/` (C0/C1 + conformance)
- `aira-schema` registry with fixture validation
- CLI: `aira schema list|validate`

## 0.1.0 — 2026-07-10

### Added

- Cargo workspace skeleton (crates + basic CSU stubs)
- Rust toolchain, rustfmt, clippy config
- GitHub Actions CI (fmt, clippy, test)
- `specs/` snapshot copies from Manifesto (Books 0–V + governance)
- README / CONTRIBUTING / SECURITY / LICENSE
- Analyze-8 bootstrap verification artifacts
