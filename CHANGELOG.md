# Changelog

## 0.1.9 — 2026-07-16

### Added

- `aira-protocol` local C2: Protocol Envelope/Response, AIRA-EP/AP adapters, Identity Descriptor, Discovery registry
- Schemas + fixtures for protocol envelope/response and identity descriptor

## 0.1.8 — 2026-07-16

### Added

- `aira-conformance` C0/C1 runners + Conformance Report Artifact
- Tests: ontology, object/artifact immutability, event causality, policy gate, pipeline, failure-to-evidence
- CLI: `aira conformance run --profile C0|C1`

## 0.1.7 — 2026-07-16

### Added

- Local `.aira` node layout (`aira init`) + Ed25519 identity create
- CLI: `problem submit|status`, `result get`, `artifact get`, `event tail`
- `aira-node` loads config/CSU registry and processes `--text`
- Persisted CAS artifact index; `run_nonce` for multi-submit safety

## 0.1.6 — 2026-07-16

### Added

- `aira-flow` OperationalPlane (problem submit + CSU event drain)
- Demos: Calculate 2+2, Ready Solution reuse, failure-to-evidence, normative split stub
- Artifact-bound CSU dispatch (`dispatch_with_artifacts`)

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
