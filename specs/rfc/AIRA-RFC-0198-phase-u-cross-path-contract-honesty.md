# AIRA-RFC-0198 — Phase U Cross-path contract honesty (closure)

## 1. Summary

Consolidating documentation atom (`#322`): Phase U `#313`–`#322` complete — cross-path contract honesty after QUEUE T closed (@ RFC-0192). Capsule-sourced verification, result-by-problem ArtifactStore authority, shared identity create fail-closed, policy audit uniqueness, durable failed submit, submit executor honesty, AddressBook selective rollback, and systemd/docs prime-port are each closed under RFC-0199…RFC-0206. Honesty invariants preserved (ArtifactStore authority; mock≠verified LLM; book≠sessions; UNKNOWN≠OFFLINE). C1 `Calculate 2 + 2` stays execution-basic. Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger implementation deps. Public bind / auto-trust remain non-default. **QUEUE U closed**; no OPEN U atoms. This atom does **not** invent a next feature phase.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
New Network tab / IA rewrite
Inventing Phase V / next feature phase in this atom
Rewriting Phase O–T history as never-DONE
Public bind / auto-trust as Desktop default
Full SQLite operational plane rewrite
Weakening observe admission hash
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| U0 Wiring | `#313` | `phase-u-plan.md`; QUEUE; `phase_u_doc`; RFC-0198 reserved file-free |
| U1 Verify | `#314` | capsule-sourced verification (RFC-0199) |
| U1 Result | `#315` | problem_id → ArtifactStore (RFC-0200) |
| U1 Identity | `#316` | shared create fail-closed (RFC-0201) |
| U1 Policy | `#317` | unique audit IDs; persist conflict (RFC-0202) |
| U1 Failed submit | `#318` | durable failure history (RFC-0203) |
| U2 Executor | `#319` | mock vs configured honesty (RFC-0204) |
| U2 AddressBook | `#320` | selective candidate rollback (RFC-0205) |
| U2 systemd | `#321` | peer bind ∈ P_AIRA (RFC-0206) |
| U Close | `#322` | this RFC; QUEUE U closed |

## 10. Per-atom contracts

```text
RFC-0199  Verification capsule-sourced (#314)
RFC-0200  Result-by-problem ArtifactStore authority (#315)
RFC-0201  CLI identity create fail-closed (#316)
RFC-0202  Policy audit uniqueness (#317)
RFC-0203  Failed submit durable (#318)
RFC-0204  Submit executor honesty (#319)
RFC-0205  AddressBook selective rollback (#320)
RFC-0206  systemd/docs prime-port (#321)
```

Id **confirmed free** 2026-09-10 (no `AIRA-RFC-0198*` in tree until this atom).

## 12. Status honesty

```text
Phase U (RFC-0198)     = cross-path contract honesty DONE
Verification           = capsule-sourced; substituted output fails (#314)
Result-by-problem      = ArtifactStore authority; index locator (#315)
Identity create        = shared fail-closed; no overwrite (#316)
Policy audit           = unique IDs; same-ID≠hash conflict (#317)
Failed submit          = durable problem + failure history (#318)
Executor honesty       = mock/reference explicit; env bind (#319)
AddressBook rollback   = own candidate only (#320)
systemd peer bind      = P_AIRA / validate_aira_port (#321)
UNKNOWN≠OFFLINE        = preserved
book≠sessions          = preserved
Global live mesh       = still PARTIAL @ N-fix (RFC-0139); unchanged by U
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_u_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger implementation deps.
