# Analyze-76 — Artifact canonical signatures (QUEUE #41)

## Status
OPEN (implementation on branch; close after merge).

## Done when
Artifact sign/verify uses canonical descriptor JSON without `signature`. Mutation of artifact descriptor fields fails verify. Payload still must match `content_hash`.

## Out
Event (already #40); Object/CSU; CAS layout.
