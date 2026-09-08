# AIRA-RFC-0164 — Phase Q Cross-install integrity & light monitoring (closure)

## 1. Summary

Consolidating documentation atom (`#285`): Phase Q `#275`–`#285` complete — Desktop production-path integrity after the post-P audit (`aira-current.md` @ `00f19cf`). Unique install-scoped identity, light model observe, root-scoped verify reopen, reachability observation independence / NAT endpoints / evidence admission, lifecycle revision races, backend≠model used, and invite import atomicity are each closed under RFC-0165…RFC-0173. C1 `Calculate 2 + 2` stays execution-basic. Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger implementation deps. **QUEUE Q closed**; no OPEN Q atoms. This atom does **not** invent Phase R.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Full Polygon mainnet as CI default
Claiming global live mesh DONE beyond N-fix honesty
Inventing Phase R / next feature phase in this atom
Rewriting Phase E–P history as never-DONE
Silent identity migration that rewrites existing install IDs
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| Q0 Wiring | `#275` | `phase-q-plan.md`; QUEUE; `phase_q_doc`; RFC-0164 reserved file-free |
| Q1 Identity | `#276` | install-scoped Desktop ID; KeyCollision (RFC-0165) |
| Q1 Model light | `#277` | observe-ready cache; admit full-hash (RFC-0166) |
| Q1 Verify context | `#278` | root-scoped reopen ready (RFC-0167) |
| Q1 Reach obs | `#279` | bind ≠ external freshness; skew (RFC-0168) |
| Q1 Reach NAT | `#280` | dial≠accept local_addr (RFC-0169) |
| Q1 Reach evidence | `#281` | root-bound; replay; apply-time (RFC-0170) |
| Q2 Lifecycle | `#282` | revision races; Quit→Stop; Applied snapshot (RFC-0171) |
| Q2 Provenance | `#283` | used ≠ backend id (RFC-0172) |
| Q2 Invite | `#284` | checks before writes; trust rollback (RFC-0173) |
| Q Close | `#285` | this RFC; QUEUE Q closed |

## 10. Per-atom contracts

```text
RFC-0165  Unique Desktop identity (#276)
RFC-0166  Model light observe (#277)
RFC-0167  Model verify context (#278)
RFC-0168  Reachability observation independence (#279)
RFC-0169  Reachability NAT endpoints (#280)
RFC-0170  Reachability evidence admission (#281)
RFC-0171  Lifecycle revision races (#282)
RFC-0172  Backend ≠ model used (#283)
RFC-0173  Invite import atomicity (#284)
```

Id **confirmed free** at `#275` (no `AIRA-RFC-0164*` in tree until this atom).

## 12. Status honesty

```text
Phase Q (RFC-0164)     = cross-install / light-monitor integrity DONE
Identity               = install-scoped; no silent TrustStore key clobber
Model observe          = light refresh; admit still full-hashes
Reachability           = independent clocks; NAT dial≠accept; root-bound apply
Lifecycle              = one revision; Quit-during-Start closed
used model             = model id/hash only; never backend:* without evidence
Invite import          = checks-before-writes; no silent trust mutation
Global live mesh       = still PARTIAL @ N-fix (RFC-0139); unchanged by Q
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_q_doc
cargo test -p aira-desktop-runtime --test invite_io
cargo clippy -p aira-desktop -- -D warnings
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger implementation deps.
