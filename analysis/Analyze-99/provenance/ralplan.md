# Analyze-99 — ralplan (QUEUE #64)

## Architect
**CLEAR** — `activate_verified` in acquisition CSU; inventory refresh only from CLI calling inventory CSU; cache under scoped `models/cache`.

## Critic
**APPROVE** — activation ≠ execution; firewall preserved; Out (sharing/rating) respected.

## Plan
1. RFC-0013 activate.
2. `activate_verified` + CLI `models activate` (+ inventory scan of cache).
3. Unit + smoke tests.
