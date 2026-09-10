# AIRA-RFC-0205 — AddressBook selective rollback (RFC-D)

## 1. Summary

Phase U `#320`: failed opt-in dial undoes only its own AddressBook candidate (reload + restore/remove that peer_id when still equal to the candidate). Parallel peer upserts survive. RFC-0198 stays file-free until `#322`.

## 2. Problem Statement

Phase T `#311` restored a full pre-dial AddressBook snapshot on failure, erasing concurrent writers during background dial wait.

## 3. Motivation

`phase-u-plan` `#320` / post-T audit §9: compensation must not clobber other peers.

## 4. Scope

- `rollback_own_dial_candidate` + `AddressBook::remove` / `endpoint_of`
- Tests: parallel upsert kept; skip when candidate already replaced
- Tip → `#321`; RFC-D **0205**

## 5. Non-Goals

```text
systemd/docs prime-port (#321)
Changing dial/Noise protocol
Merging with other U atoms
```

## 6. Compatibility / Security

Success path unchanged. Fail-closed trust/empty-addr gates unchanged.

## 7. Rollout

QUEUE `#320` → Analyze-356 → PR; next `#321` systemd prime-port.
