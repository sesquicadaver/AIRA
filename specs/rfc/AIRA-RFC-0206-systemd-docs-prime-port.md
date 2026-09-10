# AIRA-RFC-0206 — systemd/docs prime-port (RFC-D)

## 1. Summary

Phase U `#321`: `deploy/systemd/aira-peer-listen.service` and `docs/runbook-systemd.md` use a peer bind that passes `validate_aira_port` (`127.0.0.1:49157`). Living smoke rejects `:7900`. RFC-0198 stays file-free until `#322`.

## 2. Problem Statement

Runbook/unit examples used `:7900`, which fails Prime Private Port validation and crash-loops the service.

## 3. Motivation

`phase-u-plan` `#321` / post-T audit §11: executable docs must match runtime port invariant.

## 4. Scope

- Replace `:7900` with `:49157` in peer listen unit + runbook
- Living test: every `--bind` in the unit passes `validate_aira_bind`
- Tip → `#322`; RFC-D **0206**

## 5. Non-Goals

```text
RFC-0198 consolidating close (#322)
Changing HTTP aira-node listen (8787)
Rewriting peer-link DHT announce examples beyond systemd scope
Merging with other U atoms
```

## 6. Compatibility / Security

Operator may still pick another free `P_AIRA` prime; examples stay loopback.

## 7. Rollout

QUEUE `#321` → Analyze-357 → PR; next `#322` RFC-0198 close.
