# Living spec — Analyze-237 (QUEUE #202)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| B1-010 required[] on C1 2+2 body | `csu/verification-basic/src/lib.rs` | `c1.pipeline.calculate_2_plus_2`; `verified_result_body_has_b1_010_required_keys` | **DONE** |
| Completeness case uses schema required[] | `crates/aira-conformance/src/c1.rs` `missing_vra_required` | `c1.result.verified_completeness` | **DONE** |
| Capsule binds problem/context | `vra_binding_refs` | `verified_result_binds_refs_from_capsule` | **DONE** |
| RFC-0098 | `specs/rfc/AIRA-RFC-0098-vra-runtime-b1-010.md` | `phase_j_vra_runtime_202` | **DONE** |
| Event-log authority | plane `drain_from` | — | **OUT** (`#203`) |
