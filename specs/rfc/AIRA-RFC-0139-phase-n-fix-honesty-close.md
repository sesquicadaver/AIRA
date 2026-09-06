# AIRA-RFC-0139 — Phase N-fix semantic honesty & live rendezvous closure

## 1. Summary

Consolidating documentation atom (`#254`): Phase N-fix `#248`–`#254` complete. Post-N audit gaps closed without inventing Phase O. Live EVM JSON-RPC path exists (RFC-0140). Two-process ab ovo harness (RFC-0141). Reachability attestation binds Noise session transcript (RFC-0142). Expired Presence cannot promote even via `query_identity` (RFC-0143). Inbound firewall DROP ≠ non-listening placeholder (RFC-0144). `next_candidate_port` modular ring wrap (RFC-0145). Presence `validate_shape` requires `created_at < expires_at`. **Global live rendezvous status remains PARTIAL** (honest): live local/anvil JSON-RPC and honesty fixes are in; full CGNAT field trial / default public Amoy HTTPS are not claimed DONE. QUEUE N-fix closed; no OPEN N-fix atoms. RFC-0123 remains historical Phase N local-reference closure. C1 `Calculate 2 + 2` stays execution-basic. Anti-mission unchanged. `aira-core` has no ledger deps.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
AIRA-owned consensus / tokenomics
Central AIRA bootstrap as required dependency
Full CGNAT field trial as CI required
Rewriting QUEUE N history
Inventing Phase O in this atom
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| NF0 Live EVM | `#248` | JSON-RPC publish/query (RFC-0140) |
| NF1 Ab ovo 2-proc | `#249` | harness + independent roots (RFC-0141) |
| NF2 Reachability bind | `#250` | session transcript (RFC-0142) |
| NF3 Expiry promote | `#251` | fail-closed expired (RFC-0143) |
| NF4 Firewall honesty | `#252` | DROP ≠ non-listening (RFC-0144) |
| NF5 Port wrap | `#253` | modular `P_AIRA` ring (RFC-0145) |
| NF6 Temporal + close | `#254` | `created_at < expires_at`; this RFC; QUEUE N-fix closed |

## 10. Per-atom contracts

```text
RFC-0140  Live EVM rendezvous (#248)
RFC-0141  Ab ovo two-process (#249)
RFC-0142  Reachability session bind (#250)
RFC-0143  Expiry before promote (#251)
RFC-0144  Inbound firewall honesty (#252)
RFC-0145  next_candidate_port wrap (#253)
```

Id **RFC-0139** reserved at N-fix plan; file created at `#254`.

## 12. Status honesty

```text
Phase N (RFC-0123)     = local reference architecture DONE
Phase N-fix (RFC-0139) = semantic honesty + live path PARTIAL→proven locally
Global production mesh  = still PARTIAL (not claimed complete)
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_n_fix_doc
cargo test -p aira-peer --lib rejects_expires_at
cargo test -p aira-peer --lib next_candidate
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.
`aira-core` MUST NOT gain ledger/network implementation deps.
