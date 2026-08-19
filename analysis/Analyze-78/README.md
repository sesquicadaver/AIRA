# Analyze-78 — CSU manifest canonical signatures (QUEUE #43)

## Status
OPEN (implementation on branch; close after merge).

## Done when
Manifest sign/verify uses canonical JSON without `signature`. Mutation of manifest fields fails verify. Registry admission uses that verify.

## Out
Event/Artifact/Object (already #40–#42); new CSU types.
