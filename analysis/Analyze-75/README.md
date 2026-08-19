# Analyze-75 — Event canonical signatures (QUEUE #40)

## Status
CLOSED (QUEUE #40 DONE @ ad6f882 / PR #3).

## Done when
Event sign/verify uses canonical descriptor JSON without `signature`. Mutation of `event_type`, `causal_refs`, `object_refs`, `artifact_refs`, or `payload_hash` fails verify. Event log / InvariantChecker have no runtime fallback to `LOCAL_TEST_DOMAIN_MSG`.

## Out
Artifact/Object/CSU manifests; protocol envelope; Noise.
