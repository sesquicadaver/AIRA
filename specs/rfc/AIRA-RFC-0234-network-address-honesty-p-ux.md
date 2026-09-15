# AIRA-RFC-0234 — Network/address honesty + P UX (RFC-D)

## 1. Summary

QUEUE `#351` (Phase X / Pack 2 X2): Desktop Connection and Settings Connection show **honest address roles** (HTTP listen ≠ peer listen ≠ advertised discovery), **loopback peer listen** is explained as not dialable by another PC, **P0–P6** use human names (tech code kept in parentheses), and Advanced **P3 | P4** is a **radio** (mutex), not dual checkboxes.

## 2. Problem Statement

Operators mixed HTTP API bind, peer listen, and discv advertised address. Default Desktop peer listen on loopback looked like a remote dial target. P3 and P4 could be toggled as independent checkboxes despite mutex semantics. Profile chips used opaque `P0 local HTTP`-style labels.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X2: mock honesty (#350) → network/address honesty → copy/IA (#352). Parent RFC-0226 reserved until `#358`.

## 4. Scope

- Loopback hint under peer listen (System → Connection)
- Address-roles hint + labeled HTTP / peer / advertised surfaces
- Discovery announce field labeled as advertised address (`addr_advertised`)
- Human P0–P6 labels in EN/UK i18n
- Advanced: radio Base (P0–P2) | P3 | P4; leaving P3/P4 returns to P2
- Settings Connection observe-only with human profile + address roles (edit remains System → Connection)
- Help `network.connect` documents the three roles + P3|P4 radio

## 5. Non-Goals

```text
Human copy / Settings≠System IA (#352)
Help F1 model path (#353)
Compare mode (#354)
Changing peer protocol / public bind defaults
Weakening P3|P4 mutex in settings normalize
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

NetworkProfile semantics and Desktop loopback default unchanged. Settings remains observe-only for Connection; edits stay on System → Connection. `discv_addr` / `peer_listen` CLI-shaped tech keys stay EN-identical in both langs; human copy uses `addr_*` / profile Labels.

## 7. Acceptance

```text
peer listen UI → loopback hint visible when peer required
HTTP ≠ peer ≠ advertised labels/hints on Connection + Settings
P0–P6 human names in EN/UK
Advanced P3|P4 is radio (not simultaneous checkboxes)
Tip → first OPEN #352
```

## 8. References

- QUEUE `#351` · Analyze-388
- Parent: RFC-0226; prior RFC-0233 mock honesty; desktop-ux §8
