# AIRA-RFC-0146 — Phase O Desktop UX contract & contextual Help (F1) closure

## 1. Summary

Consolidating documentation atom (`#265`): Phase O `#255`–`#265` complete — Desktop UX contract and offline contextual Help (F1). Shell IA is **Work / System / Settings** plus persistent **Help · F1**. `SystemSnapshot` honesty keeps UNKNOWN≠OFFLINE and AddressBook≠live sessions. Work submit/refresh stay off the egui paint path. Stable action/error/help IDs share a lexicon with Labels. Settings surface Saved≠Applied≠Restart needed. Offline Help embeds `docs/help/{en,uk}` with search, context routing, and `help:<id>` link check. C1 `Calculate 2 + 2` stays execution-basic. Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/Help/egui deps. **QUEUE O closed**; no OPEN O atoms. This atom does **not** invent Phase P.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Required central help CDN
WCAG formal certification as merge gate
Inventing Phase P / next feature phase in this atom
Rewriting Phase E–N history as never-DONE
Claiming global live rendezvous beyond N-fix honesty (stays PARTIAL)
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| O0 Wiring | `#255` | `phase-o-plan.md`; `desktop-ux` canon; `phase_o_doc`; RFC-0146 reserved |
| O1 Snapshot | `#256` | SystemSnapshot honesty (RFC-0147) |
| O2 Non-blocking | `#257` | async submit/refresh (RFC-0148) |
| O3 IDs | `#258` | action/error/help lexicon (RFC-0149) |
| O4 Shell IA | `#259` | Work/System/Settings + Help·F1 (RFC-0150) |
| O5 Work | `#260` | user-language; draft; Ctrl+Enter; provenance (RFC-0151) |
| O6 System | `#261` | Program/Model/Connection/Events (RFC-0152) |
| O7 Settings | `#262` | Saved≠Applied≠Restart (RFC-0153) |
| O8 F1 shell | `#263` | embed/search/context offline Help (RFC-0154) |
| O9 Topics | `#264` | uk/en seed + `help:` link check (RFC-0155) |
| O10 Close | `#265` | this RFC; QUEUE O closed |

## 10. Per-atom contracts

```text
RFC-0147  SystemSnapshot honesty (#256)
RFC-0148  Non-blocking work/refresh (#257)
RFC-0149  Action/error/help IDs (#258)
RFC-0150  Shell IA + Help·F1 chrome (#259)
RFC-0151  Work screen provenance (#260)
RFC-0152  System status screen (#261)
RFC-0153  Settings apply lifecycle (#262)
RFC-0154  Offline F1 shell (#263)
RFC-0155  Seed help topics + link check (#264)
```

Id **confirmed free** at `#255` (no `AIRA-RFC-0146*` in tree until this atom).

## 12. Status honesty

```text
Phase O (RFC-0146)     = Desktop Developer Preview + honest end-user shell DONE
Help F1                = offline embedded catalog; not a CDN / LLM tutor
Global live mesh       = still PARTIAL @ N-fix (RFC-0139); unchanged by O
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_o_doc
cargo test -p aira-desktop
cargo clippy -p aira-desktop -- -D warnings
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger implementation deps.
