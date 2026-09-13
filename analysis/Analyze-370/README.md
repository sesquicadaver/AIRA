# Analyze-370 — Strict submit decoding (`#333`)

**Status:** DONE @ RFC-0217  
**Done when:** unknown nested/top-level submit fields → 4xx before execution; text-only OK.

## Evidence

- `crates/aira-flow/src/admission.rs` — `deny_unknown_fields` on request constraint structs
- `crates/aira-node/src/http/handlers.rs` — `ProblemSubmitBody` deny unknown
- Tests: `admission::tests::constraints_deny_*`; `http_post_problem_unknown_*_is_4xx`
