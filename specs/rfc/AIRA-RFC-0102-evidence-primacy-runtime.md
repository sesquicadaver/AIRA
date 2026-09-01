# AIRA-RFC-0102 — Evidence primacy runtime (Claim vs Assumption)

## 1. Summary

Phase J `#206`: `OperationalPlane` rejects a resolved artifact whose JSON body has `claim_kind: Claim` and no non-empty `evidence_refs` (`FlowError::EvidencePrimacy`) before log append. `Assumption` and `Hypothesis` may omit evidence. Schema fixtures `#125` / RFC-0075 unchanged. Does not import a second CSU into evidence-basic beyond the helper.

## 5. Non-Goals

Epistemic emit on C1 (`#207`); full Epistemic plane; RFC-0096 (`#208`); changing `claim-artifact.schema.json`.

## 10. Contract

```text
claim_kind == Claim AND no non-empty evidence_refs
  → EvidencePrimacy (no append), CAS publish still allowed
claim_kind == Assumption | Hypothesis (empty evidence_refs) → operational
body without claim_kind → not a claim coordinate (VRA / capsule OK)
```

## 15. Tests

```text
cargo test -p aira-csu-evidence-basic --lib claim_without_evidence_lacks_required_evidence
cargo test -p aira-flow --lib claim_without_evidence_rejected_as_operational_input
cargo test -p aira-flow --lib assumption_without_evidence_is_operational_input
```
