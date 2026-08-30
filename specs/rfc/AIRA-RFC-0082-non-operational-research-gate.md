# AIRA-RFC-0082 — Non-operational research gate (OperationalPlane)

## 1. Summary

Phase H `#179`: `OperationalPlane` rejects Book V research types as operational input. `ResearchArtifactCreated` / `ArtifactPromotionCandidate` events, and events whose `artifact_refs` resolve to `ResearchArtifact` / `OpenResearchArtifact` / `BestCurrentHypothesisArtifact`, return `FlowError::ResearchNonOperational` **before** log append. CAS publish of research artifacts remains allowed. Not `run_c5`.

## 5. Non-Goals

`run_c5` (`#180`); promotion status rollup (`#181`); promoting any research item; Core/ABI change.

## 10. Behavior

```text
inject_and_drain(event)
  event_type.is_research_until_promoted → ResearchNonOperational (no append)
  resolved artifact_refs research-until-promoted → ResearchNonOperational (no append)
drain_from: same check before CSU dispatch
Unresolved `artifact_refs` do **not** trip this gate (operational missing-ref paths such as CapsuleCreated → CapsuleFailed stay intact). Only **resolved** research ArtifactTypes are rejected.
```

## 15. Tests

```text
cargo test -p aira-flow research_artifact_rejected_as_operational_input
cargo test -p aira-artifact research_types_are_non_operational
cargo test -p aira-event research_event_types_are_non_operational
```
