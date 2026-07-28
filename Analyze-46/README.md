# Analyze-46 — Partial C2 conformance (M13)

**Scope:** Formal `run_c2` suite for Roadmap M13 exit criteria. Wire CLI/HTTP. No DHT, no network C2, no Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## RALPLAN-DR

### Principles
1. Gate existing local protocol code with conformance, don’t reinvent adapters
2. One profile slice: C2 local only
3. Failures must be evidence-friendly (standard report artifact)
4. M12 C0/C1 remain unchanged
5. No federation / DHT

### Decision Drivers
1. QUEUE #12 + roadmap M13 exit criteria
2. C2 enum already present but unimplemented in `run_profile`
3. Fixtures + adapters already ship

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. `run_c2` suite over fixtures + adapters** | Matches M13 exit; small | Doesn’t add new protocols |
| B. Full network C2 | Completeness | Out of MVP / anti-pattern |
| C. Docs-only claim M13 done | Zero code | No harness gate |

**Chosen: A.**

### Architect
- **Antithesis:** Expanding C1 vs new profile — C2 semantics differ (protocols).
- **Tension:** Harness vs new features — synthesis: harness-first closes M13 gate.
- **Critic: APPROVE**

### Acceptance
- [ ] `run_c2` passes with ≥5 cases covering M13 exits
- [ ] `run_profile(C2)` works
- [ ] CLI + HTTP accept C2
- [ ] docs/conformance.md + QUEUE
- [ ] Tests + clippy; CODE_REVIEW APPROVE/CLEAR

### Out
Network C2, DHT, federation, security/alpha profile expansion.
