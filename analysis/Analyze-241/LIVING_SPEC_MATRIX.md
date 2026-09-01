# Living spec — Analyze-241 (QUEUE #206)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Claim without evidence rejected at runtime | `crates/aira-flow/src/plane.rs` `reject_claim_without_evidence` | `claim_without_evidence_rejected_as_operational_input` | **DONE** |
| Assumption may omit evidence | same | `assumption_without_evidence_is_operational_input` | **DONE** |
| Claim with evidence allowed | same | `claim_with_evidence_is_operational_input` | **DONE** |
| Helper | `csu/evidence-basic` `claim_lacks_required_evidence` | `claim_without_evidence_lacks_required_evidence` | **DONE** |
| RFC-0102 | `specs/rfc/AIRA-RFC-0102-evidence-primacy-runtime.md` | `phase_j_evidence_primacy_206` | **DONE** |
| Epistemic emit C1 | epistemic-basic | — | **OUT** (`#207`) |
