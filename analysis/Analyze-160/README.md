# Analyze-160 — Claim/Assumption fixtures (QUEUE #125)

## Status
OPEN — branch `queue-125-claim-assumption-fixtures`.

## Done when
`schemas/evidence/claim-artifact.schema.json`; valid claim + assumption fixtures; invalid claim-without-evidence + missing-signature; `fixtures/manifest.json` entries; `schema validate --fixtures` green; B0-005 partial closed.

## Out
Epistemic CSU implementation (#141).

## Living spec

| Requirement | Artifact | Gate |
|-------------|----------|------|
| B0-005 Claim evidence primacy | `claim-artifact.schema.json` `allOf` if/then | invalid `claim-artifact-no-evidence.json` |
| Book 0 §6.2 coordinates | required fields on schema | valid fixtures |
| Assumption without evidence | `claim_kind` Assumption | `assumption-artifact.json` |
