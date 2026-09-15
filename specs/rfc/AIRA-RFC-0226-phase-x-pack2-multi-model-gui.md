# AIRA-RFC-0226 — Phase X Pack 2 multi-model GUI (closure)

## 1. Summary

Consolidating documentation atom (`#358`): Phase X `#343`–`#358` complete — Pack 2 local multi-model GUI after QUEUE W closed (@ RFC-0215) and GUI sources `aira-current.md` (2026-09-13) + Pack 2 / audit M1–M6. Per-model inventory lifecycle, select API Auto/required, request profile → immutable snapshot, CLI supported-contract parity, Settings Models catalog GUI, Work executor + capability readiness, mock honesty + result triple, network/address honesty + P UX, human copy + Settings≠System IA, Help F1 model path, Work Compare (no silent substitute), model data paths, apply-diff honesty, and two-model installed acceptance are each closed under RFC-0227…RFC-0240. Honesty invariants preserved (choice never grants VERIFIED; mock remains explicit; admission/activation evidence not weakened for UX). Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger/LLM runtime deps. Public bind / auto-trust remain non-default. Pack 3+ ratings / remote compute / weight-transfer remain separate (no new QUEUE IDs here). **QUEUE X closed**; no OPEN X atoms. This atom does **not** invent Pack 3–7 / Phase Y.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Pack 3 ratings / evidence measurement UI
Pack 4 remote compute + OS quota product
Pack 5 weight transfer path ≠ activated publish
Pack 6–7 federation / product profile
Rewriting Phase W as never-DONE / inventing Phase Y here
Public bind / auto-trust as Desktop default
Weakening observe / activate admission hash
Minting VERIFIED for model choice
Inventing first OPEN #359 / Pack 3 QUEUE band
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| X0 Wiring | `#343` | `phase-x-plan.md`; QUEUE; `phase_x_doc`; RFC-0226 reserved file-free |
| X1 Inventory (M1) | `#344` | A/B independent; latest ≠ sole auth (RFC-0227) |
| X1 Select API (M2) | `#345` | Auto + required; unready explained (RFC-0228) |
| X1 Profile→snapshot (M3) | `#346` | overrides/excludes; mid-flight safe (RFC-0229) |
| X1 CLI parity (M4) | `#347` | no no-op flags (RFC-0230) |
| X2 Settings Models | `#348` | list/scan/add/select/prepare + ready reason (RFC-0231) |
| X2 Work executor | `#349` | Auto/specific; math≠generate; pre-submit (RFC-0232) |
| X2 Mock + result | `#350` | demo banner; requested/applied/executed (RFC-0233) |
| X2 Network/P UX | `#351` | loopback; HTTP≠peer≠advertised; P3\|P4 (RFC-0234) |
| X2 Copy + IA | `#352` | i18n; Settings edit ≠ System observe (RFC-0235) |
| X2 Help F1 | `#353` | UK/EN offline model path (RFC-0236) |
| X2 Compare | `#354` | dual model; no silent substitute (RFC-0237) |
| X2 Data paths | `#355` | storage catalog + space (RFC-0238) |
| X2 Apply-diff | `#356` | Changed/Saved/Applied/Restart + error (RFC-0239) |
| X3 M6 acceptance | `#357` | GUI+CLI+HTTP; cold restart; fail-closed (RFC-0240) |
| X Close | `#358` | this RFC; QUEUE X closed; Pack 2 DONE |

## 10. Per-atom contracts

```text
RFC-0227  Per-model inventory lifecycle (#344)
RFC-0228  Model select API Auto/required (#345)
RFC-0229  Request profile → snapshot (#346)
RFC-0230  CLI supported-contract parity (#347)
RFC-0231  Settings Models catalog GUI (#348)
RFC-0232  Work executor + capability readiness (#349)
RFC-0233  Mock honesty + result triple (#350)
RFC-0234  Network/address honesty + P UX (#351)
RFC-0235  Human copy + Settings≠System IA (#352)
RFC-0236  Help F1 model path (#353)
RFC-0237  Work Compare mode (#354)
RFC-0238  Model data paths (#355)
RFC-0239  Apply-diff honesty (#356)
RFC-0240  Two-model installed acceptance (#357)
```

Id **confirmed free** 2026-09-13 (no `AIRA-RFC-0226*` in tree until this atom).

## 12. Status honesty

```text
Phase X (RFC-0226)     = Pack 2 local multi-model GUI DONE
Inventory lifecycle    = A/B independent; latest ≠ sole auth (#344)
Select API             = Auto/required; explained unready (#345)
Profile→snapshot       = freeze allowed/excluded (#346)
CLI parity             = supported flags only (#347)
Settings Models        = catalog without CLI (#348)
Work executor          = readiness by capability (#349)
Mock + result          = demo banner; triple honesty (#350)
Network/P UX           = address roles + P3|P4 (#351)
Copy + IA              = Settings edit ≠ System observe (#352)
Help F1                = offline model path (#353)
Compare                = dual model; no silent substitute (#354)
Data paths             = storage catalog + space (#355)
Apply-diff             = phase honesty + apply error (#356)
M6 acceptance          = GUI+CLI+HTTP; cold restart (#357)
Pack 3+                = still separate (no QUEUE IDs here)
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_x_doc
cargo test -p aira-desktop-runtime --test phase_w_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger / LLM implementation deps.

## 16. Handoff

```text
QUEUE X closed @ RFC-0226
  → no OPEN X atoms
  → Pack 3+ still separate (do not invent #359 / Phase Y here)
```
