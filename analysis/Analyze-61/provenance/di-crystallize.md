# DI crystallize — Analyze-61 / QUEUE #26

**Chosen:** **C** — both `--keep N` and `--older-than-days D` (at least one required).
**Scope:** archived stamp slots for **ed25519** (`local.ed25519.prev.<stamp>`) **and** **x25519** (`local.x25519.prev.<stamp>`).
**Invariant:** never delete canonical latest `.prev` / `.prev.meta.json`.
**CLI:** `identity backups prune …` (list remains `identity backups`).
**Recommended:** keep=3 or days=31 (docs; not hard-coded defaults).
**Out:** per-CSU secrets (#27); auto-prune on rotate (explicit CLI only).
