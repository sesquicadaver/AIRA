# Ralplan — Analyze-59 (QUEUE #24) — revision 1 (consensus)

**DI:** Option **C**. **Architect:** APPROVE. **Critic:** APPROVE.

## Principles
1. Accept loop must not await handshake or recv (daemon mode, including `--relay`)
2. Keep `accept()` as composed TCP+handshake for tests / `--once`
3. Fail-closed handshake errors: per-task log + drop; listener stays up
4. Discovery / “accepted” / relay register **only after** successful `complete_accept`
5. No wire/Noise/trust protocol changes; no systemd (#25); no Manifesto/Meditation edits

## Decision Drivers
1. QUEUE #24 Done when: ≥2 parallel sessions without blocking accept loop
2. User chose C (handshake off accept loop)
3. Minimal API split + CLI spawn earlier

## Options
| Option | Verdict |
|--------|---------|
| A test-only | Rejected (user / does not fix handshake HOL) |
| B persistent recv-loop | Deferred (not this row) |
| **C split accept_tcp / complete_accept + spawn** | **Chosen** |

## API
- `accept_tcp(listener) -> TcpStream` — unbounded TCP wait
- `complete_accept(stream, root) -> AuthenticatedPeer` — hello + Noise (timed)
- `accept(listener, root)` = accept_tcp + complete_accept
- Daemon `!once` (recv **and** relay): `accept_tcp` → spawn `{ complete_accept → side effects → serve/recv }`
- `--once`: sequential composed `accept` (or inline complete)

## Test-spec
1. Hung: raw TCP no-hello; parallel real dial succeeds (accept not blocked)
2. Broken/corrupt or timeout handshake on one conn; listener still accepts next dial
3. ≥2 parallel authenticated sessions both recv (overlap)
4. Existing tests via composed `accept` still pass
5. Relay daemon path uses the same non-blocking accept_tcp pattern (smoke or unit)

## Out / WATCH
- Out: systemd; multi-envelope session loop; STUN; discv5; wire change
- WATCH (non-blocking): optional handshake concurrency semaphore (future)
