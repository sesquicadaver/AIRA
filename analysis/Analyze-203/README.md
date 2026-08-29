# Analyze-203 — CRP multi-candidate gate (QUEUE #168)

## Done
- `LocalCrpAdapter::route` emits ≥2 candidates when multiple CSU providers exist
- `bind(..., PolicyGate)` requires ALLOW on `crp.bind`; DENY → no bind
- RFC-0079 updated; test `crp_multi_candidate_and_policy_gate_bind`
- QUEUE `#168` DONE → `#169` OPEN

## Out
Route events (`#169`); B2-006 C3 case (`#170`).
