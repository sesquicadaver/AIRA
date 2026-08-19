# Analyze-9 — Schema Registry + Terminology

**Цикл:** Analyze-9  
**Scope:** Issue #8 + Epic 2 (#9–#21)

## Ralplan (approved)

### Principles
1. Schemas з Schema Pack; без forbidden couplings (gpu_id, node_id as core, …)
2. Originals immutable; `specs/` / `docs/` / `schemas/` — робочі артефакти
3. Validation machine-checkable через `aira-schema` + fixtures
4. MVP freeze: не runtime C0

### Decision
Повний Epic 2 для C0/C1 мінімуму Schema Pack §31 + conformance report; terminology guardrail у `docs/canonical-terminology.md`.

### Acceptance
- `docs/canonical-terminology.md`
- schema dirs + JSON files
- `cargo test -p aira-schema` valid/invalid fixtures
- `aira schema list|validate`
- deny-originals PASS
