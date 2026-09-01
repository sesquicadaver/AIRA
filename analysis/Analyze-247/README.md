# Analyze-247 — Reduction bind (QUEUE #212)

## Done
- `catalog_action` in `csu/reduction-basic`: echo / uppercase unchanged; `Calculate 2 + 2` → `math.eval.safe`; other prose → `text.generate.local`
- Generate-local capsule is RFC-0105 CustomArtifact (`aira:schema:execution:generate-local:0.1`); constraints `network=none`, `shell=false`
- No Cargo dep on `aira-csu-execution-llm` (CSU ↛ CSU)
- Named tests: `calculate_2_plus_2_binds_math_eval_safe`; `non_math_prompt_binds_generate_local`; `echo_and_uppercase_keep_existing_binds`
- RFC-D [`AIRA-RFC-0107`](../../specs/rfc/AIRA-RFC-0107-reduction-generate-local.md)
- QUEUE `#212` **DONE**; first OPEN `#213`
- RFC-0104 remains file-free (`phase_k_rfc_0104_id_free`)

## Out
Plane register (`#213`); activate policy (`#214`); process backend (`#215`); Desktop Work generate; RFC-0104 (`#216`); Core inference host; changing execution-basic math path.
