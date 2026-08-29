# Analyze-200 — CRP schema fixtures (QUEUE #165)

## Done
- Schemas: `aira:schema:protocol:crp-route-request:0.1`, `aira:schema:protocol:crp-route-candidate:0.1` (Book II §10.2–10.3)
- Valid + invalid fixtures registered in `fixtures/manifest.json`
- `cargo run -p aira-cli --locked -- schema validate --fixtures fixtures` → 64 passed
- QUEUE `#165` DONE → `#166` OPEN; CRP runtime status remains **ABSENT** until `#171`

## Out
In-process CRP adapter + RFC (`#166`); node-keyed reject; B2-006 C3 case.
