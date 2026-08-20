# Analyze-101 — ralplan (QUEUE #66)

## Architect
**CLEAR** — extend acquisition CSU with publish gate (policy already has `share_custom_models`); no new CSU crate yet; network=none.

## Critic
**APPROVE** — mirrors download gate; Out (`#67` publish) respected.

## Plan
1. RFC-0015 share gate.
2. `request_publish` + policy write flag + CLI.
3. Unit + smoke tests.
