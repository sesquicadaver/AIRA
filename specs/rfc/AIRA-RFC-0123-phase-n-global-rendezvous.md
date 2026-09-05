# AIRA-RFC-0123 — Phase N Global Node Rendezvous closure

## 1. Summary

Consolidating documentation atom (`#247`): Phase N `#231`–`#247` complete — Global Node Rendezvous & Prime Connectivity. AIRA-owned peer/discv/relay endpoints use only the Prime Private Port set `P_AIRA` (|P_AIRA|=1491). Preferred port is deterministic from identity. Presence Records are canonical-signed. Rendezvous is ledger-agnostic (`RendezvousProvider`); local-file and EVM local-double adapters live outside `aira-core`. Reachability is peer-assisted (no hairpin DIRECT). AddressBook promotion is trust-gated (`DISCOVERED ≠ TRUSTED`). Dial order is direct → NAT observed → relay courier. CLI/Desktop orchestrate APIs only. Ab ovo and inbound-blocked NAT/relay integration smokes pass. C1 `Calculate 2 + 2` stays `math.eval.safe` / execution-basic. Anti-mission (GPU marketplace / LLM-in-Core / AIRA-owned consensus) unchanged. QUEUE N closed; no OPEN N atoms.

## 5. Non-Goals

```text
GPU marketplace
LLM-in-Core / LLM runtime in aira-core
AIRA-owned blockchain / tokenomics / consensus
Central AIRA bootstrap server as required dependency
Auto-trust from ledger discovery
TCP Noise tunnel proxy through relay (courier model retained)
Live public EVM JSON-RPC dial as CI default
Inventing Phase O in this atom
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| N0 govern | `#231` | `phase-n-plan.md`; living `phase_n_doc` |
| N1 Prime Port | `#232` | `P_AIRA`; fail-closed (RFC-0124) |
| N2 selection | `#233` | `preferred_port`; collision walk (RFC-0125) |
| N3 Presence | `#234` | schema + canonical Ed25519 (RFC-0126) |
| N4 trait | `#235` | `RendezvousProvider` + mock (RFC-0127) |
| N5 EVM adapter | `#236` | local double; Amoy/mainnet hooks (RFC-0128) |
| N6 publish/query | `#237` | TTL/sequence + rendezvous.json (RFC-0129) |
| N7 probe | `#238` | peer-assisted challenge (RFC-0130) |
| N8 states | `#239` | UNKNOWN…OFFLINE; reachability.json (RFC-0131) |
| N9 promote | `#240` | trust-gated Presence→AddressBook (RFC-0132) |
| N10 relay path | `#241` | direct→NAT→relay; ads; dual reservation (RFC-0133) |
| N11 refresh | `#242` | sequence++; expire; endpoint change (RFC-0134) |
| N12 CLI | `#243` | peer port/reachability/rendezvous (RFC-0135) |
| N13 Desktop | `#244` | Network mesh panel (RFC-0136) |
| N14 Ab ovo | `#245` | publish→discover→trust→dial (RFC-0137) |
| N15 NAT/relay | `#246` | inbound blocked → courier (RFC-0138) |
| N16 close | `#247` | this RFC; QUEUE N closed |

## 10. Per-atom contracts

```text
RFC-0124  Prime Port (#232)
RFC-0125  preferred port (#233)
RFC-0126  Presence Record (#234)
RFC-0127  RendezvousProvider (#235)
RFC-0128  EVM rendezvous adapter (#236)
RFC-0129  publish/query (#237)
RFC-0130  Reachability Probe (#238)
RFC-0131  Reachability states (#239)
RFC-0132  AddressBook promotion (#240)
RFC-0133  Relay dial path (#241)
RFC-0134  Presence refresh (#242)
RFC-0135  Phase N CLI (#243)
RFC-0136  Desktop Network mesh (#244)
RFC-0137  Ab ovo integration (#245)
RFC-0138  NAT/relay integration (#246)
```

Id **confirmed free** at `#231` (no `AIRA-RFC-0123*` in tree until this atom).

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_n_doc
cargo test -p aira-peer --lib ab_ovo
cargo test -p aira-peer --lib nat_relay
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.
`aira-core` MUST NOT gain ledger/network implementation deps.
