# Provenance — Artifact / Event / Policy scope

## Delivered
Epic 4 (#27–#34):
- `aira-artifact`: descriptor + filesystem CAS store + supersession
- `aira-event`: descriptor + `EventSink` / `EventLog` + `MemoryEventLog` subscriptions
- `aira-policy`: `PolicyGate` ALLOW|DENY|REQUIRE + `PolicyEvaluated` events
- `aira-core`: expanded `InvariantViolation` + `InvariantChecker`

## Explicit non-goals this cycle
- CSU runtime / dispatch (Epic 5)
- Crypto verify of signatures (structural only)
- Persistent event log / durable artifact index across process restarts beyond CAS blobs
- Claiming AIRA-C0 conformance harness pass

## Originals
`Manifesto etc/` and `Meditation_About/` unchanged.
