# Analyze-34 — Peer listen daemon

**Scope:** Persistent multi-accept `peer listen` + dial smoke without mandatory envelope recv.

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. One-shot listen must not block peer dial smoke (hello-only accept)
2. Daemon listen accepts many peers sequentially (or concurrent recv tasks)
3. TCP `accept` wait must not use the 10s peer I/O deadline (idle listen is valid)
4. No wire/hello/Noise/NAT changes; loopback default remains
5. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### Decision Drivers
1. QUEUE #1 P0 before Noise
2. Minimal surface: CLI flags + fix accept idle timeout
3. Fail closed trust gate unchanged

### Options
| Option | Pros | Cons |
|--------|------|------|
| **A. CLI loop + `--once`/`--recv`** | Small diff, matches A-33 | No library serve API |
| B. Full `aira-peer::serve` service | Reusable | Over-scope for P0 |

**Chosen: A.** Invalidates B for this cycle (follow-up if Noise needs shared serve).

### ADR
- **Decision:** `peer listen` defaults to persistent accept loop; `--once` for single accept; `--recv` optional envelope receive (spawned per connection in daemon mode). Remove idle timeout from TCP `accept` wait; keep handshake/frame timeouts.
- **Why:** Unblocks dial smoke and multi-dial demos without crypto expansion
- **Reject:** Noise, trust-delta, multi-threaded connection pool API
- **Follow-up:** Noise XX (QUEUE #2)

### Acceptance
- [x] `accept` waits indefinitely for next TCP connection (handshake still timed)
- [x] `peer listen` loops until SIGINT / `--once`
- [x] Without `--recv`, dial-only hello succeeds while listener is up
- [x] With `--recv`, listener prints received envelope(s)
- [x] Multi-accept: two dials against one listener both succeed
- [x] Tests + clippy/fmt; docs + Analyze-34 + QUEUE update
