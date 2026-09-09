# AIRA-RFC-0197 — Dial AddressBook candidate honesty (RFC-D)

## 1. Summary

Phase T `#311`: opt-in peer dial may trial-upsert an AddressBook candidate for the attempt, but a **failed** dial restores pre-attempt book authority. A known-good address A is not silently replaced by a failed trial B. Successful dial keeps the candidate as dial authority. RFC-0192 stays file-free until `#312`.

## 2. Problem Statement

`run_opt_in_peer_dial` wrote AddressBook **before** `aira_peer::dial`. On connect/handshake failure the book already held the candidate endpoint, so operators lost the prior authoritative address without an explicit save.

## 3. Motivation

`phase-t-plan` `#311` / post-S audit §6: failed probe must not quietly overwrite AddressBook. Chosen contract: **rollback on fail** (trial upsert for dial lookup, restore prior snapshot on error). Explicit separate “save address” UI is out of scope.

## 4. Scope

- Snapshot `AddressBook` before trial upsert; restore via `prior.save` on dial failure
- Error text states restoration (or dual-failure if rollback itself fails)
- Tests: prior A survives failed B; no-prior failed dial leaves no orphan entry
- Docs / Help / QUEUE tip → `#312`
- RFC-D **0197**

## 5. Non-Goals

```text
RFC-0192 consolidating close (#312)
Separate “save address” chrome without dial
Changing AddressBook as dial authority
Merging with dial live-honesty / observe / Quit atoms
```

## 6. Compatibility / Security

Dial still requires trust + explicit address. Upsert still validates P_AIRA ports. Reachability DIRECT is unchanged. Successful dial evidence path unchanged.

## 7. Rollout

QUEUE `#311` → Analyze-347 → PR; next `#312` RFC-0192 + QUEUE T close.
