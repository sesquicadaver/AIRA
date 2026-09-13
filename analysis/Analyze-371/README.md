# Analyze-371 — Constraints enforce-or-reject (`#334`)

**Status:** DONE @ RFC-0218

## Evidence

- `AdmissionSnapshot::enforce_or_reject`
- `FlowError::UnsupportedConstraint` → HTTP 400
- Tests: `enforce_*`, `http_post_problem_math_model_ref_is_4xx`, `http_post_problem_remote_required_is_4xx`
