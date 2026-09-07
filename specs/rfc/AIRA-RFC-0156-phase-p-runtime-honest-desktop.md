# AIRA-RFC-0156 — Phase P Runtime-honest Desktop (closure)

## 1. Summary

Consolidating documentation atom (`#274`): Phase P `#266`–`#274` complete — Desktop UI labels match verified runtime facts after the post-O audit. Snapshot freshness (`DataQuality::Stale`), Applied vs Explicitly Undefined, model selected≠ready≠used, reachability endpoint+direction bind, EVM/JSON-RPC PARTIAL honesty, non-blocking Start/Stop/Quit, and Help·F1 focus→section→screen without stale-search hijack are each closed under RFC-0157…RFC-0163. C1 `Calculate 2 + 2` stays execution-basic. Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger implementation deps. **QUEUE P closed**; no OPEN P atoms. This atom does **not** invent Phase Q.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Claiming Polygon on-chain ledger success from Mock/reference HTTP
Claiming global live rendezvous beyond N-fix honesty (stays PARTIAL)
Inventing Phase Q / next feature phase in this atom
Rewriting Phase E–O history as never-DONE
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| P0 Wiring | `#266` | `phase-p-plan.md`; QUEUE; `phase_p_doc`; RFC-0156 reserved |
| P1 Freshness | `#267` | Snapshot Stale / measurement≠load (RFC-0157) |
| P1 Applied | `#268` | Applied from confirmed runtime (RFC-0158) |
| P1 Model | `#269` | selected≠ready≠used (RFC-0159) |
| P1 Reachability | `#270` | endpoint+direction evidence (RFC-0160) |
| P1 EVM | `#271` | https config; Mock≠ledger; PARTIAL (RFC-0161) |
| P2 Lifecycle | `#272` | Start/Stop/Quit off update() (RFC-0162) |
| P2 Help | `#273` | F1 routing; clear search; titles (RFC-0163) |
| P Close | `#274` | this RFC; QUEUE P closed |

## 10. Per-atom contracts

```text
RFC-0157  Snapshot freshness (#267)
RFC-0158  Applied from runtime (#268)
RFC-0159  Model triple UX (#269)
RFC-0160  Reachability endpoint bind (#270)
RFC-0161  EVM/JSON-RPC honesty (#271)
RFC-0162  Lifecycle non-blocking (#272)
RFC-0163  Help F1 routing (#273)
```

Id **confirmed free** at `#266` (no `AIRA-RFC-0156*` in tree until this atom).

## 12. Status honesty

```text
Phase P (RFC-0156)     = runtime-honest Desktop labels DONE
Reachability           = endpoint-bound local evidence; not transcript-alone
EVM / JSON-RPC         = PARTIAL reference/Mock; not Polygon ledger proof
Global live mesh       = still PARTIAL @ N-fix (RFC-0139); unchanged by P
Help F1                = focus→section→screen; offline embedded; not LLM tutor
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_p_doc
cargo test -p aira-desktop
cargo clippy -p aira-desktop -- -D warnings
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger implementation deps.
