# AIRA-RFC-0069 — Phase G Reference completion v0.2 closure

## 1. Summary

Consolidating documentation atom (`#151`): Phase G `#120`–`#151` complete — reference posture **v0.2** documented in `README.md`, `docs/implementation-status.md`, `docs/phase-g-plan.md`, and this RFC. Anti-mission (GPU marketplace / LLM runtime / blockchain / scheduler) unchanged.

## 5. Non-Goals

Post-G product pivot; CRP/settlement/PHM as runtime; App Store / enterprise Authenticode beyond packaging helpers; new Phase H backlog without a separate plan.

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| G0 govern | `#120`–`#121` | branch protection truth; living spec smoke |
| G1 C2 deepen | `#122`–`#124` | idempotency; artifact hash; unsigned envelope |
| G2 evidence | `#125`–`#126` | Claim/Assumption fixtures; VRA extended fields |
| G3 modularize | `#127`–`#130` | crypto / peer / cli peer / desktop splits |
| G4 desktop peer | `#131`–`#133` | P3/P4 CI stable; invite QR camera |
| G-SEC | `#134`–`#138` | local-test trust; envelope sign; identity bind; equivocation; artifact admission |
| G5 federation/C3 | `#139`–`#141` | leave; join hardening; `run_c3` scaffold |
| G6 stores | `#142`–`#143` | event log recovery; SQLite migrate/integrity |
| G7 CSU SDK | `#144`–`#145` | external fixture; `docs/csu-sdk.md` |
| G8 epistemic | `#146`–`#147` | `epistemic-basic`; plane/session assessment path |
| G9 packaging | `#148`–`#150` | macOS notarize helper; Windows MSI stage; Linux deb |
| G10 docs | `#151` | this RFC + README / implementation-status v0.2 |

## 15. Tests

`cargo test -p aira-desktop-runtime --test phase_g_doc`
