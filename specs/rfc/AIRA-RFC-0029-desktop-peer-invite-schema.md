# AIRA-RFC-0029 — Desktop PeerInvite payload schema

## 1. Summary

Additive JSON Schema `aira:schema:desktop:peer-invite:0.1` describes a friend-onboarding invite (identity_ref + Ed25519 pubkey + optional dial addr) for file/QR exchange in Addendum E1.1. Not a Core entity.

## 2. Problem Statement

QUEUE `#80` needs a fixed invite document so `#83`/`#84` file+QR and `#85` GUI share one contract before trust/book apply.

## 3. Motivation

[`docs/phase-e-plan.md`](../../docs/phase-e-plan.md) §4a and [`docs/desktop-ux.md`](../../docs/desktop-ux.md) §3 (P1 onboarding via file/QR).

## 4. Scope

- `schemas/desktop/peer-invite.schema.json`
- Valid/invalid fixtures + `fixtures/manifest.json`
- Unit coverage in `aira-schema`

## 5. Non-Goals

```text
settings P1 / peer_listen (#81)
peer process supervise (#82)
export/import / trust+book apply (#83)
QR PNG (#84)
GUI (#85)
P2–P6 / DHT / camera
Book 0 / aira-core / C1 change
```

## 6. Current Behavior

No `aira:schema:desktop:peer-invite:*`. Friend onboarding uses ad-hoc CLI trust/book only.

## 7. Proposed Change

Required:

```text
payload_schema = aira:schema:desktop:peer-invite:0.1
identity_ref   = aira:schema:common:ref:0.1
public_key_hex = 64 hex chars (Ed25519 verifying key)
```

Optional: `addr` (string|null host:port), `display_name`, `created_at`. `additionalProperties: false`. Missing `identity_ref` MUST fail.

## 8. Affected Books / Schemas / Tests

- Schema Pack: additive desktop schema
- Fixtures + `aira-schema` unit test + `schema validate --fixtures`
- Books 0–III: none

## 9. Compatibility Impact

Additive.

## 10. Security Impact

Schema only. Does not auto-trust; apply is `#83`. Pubkey is verifying material intended for explicit import.

## 11. Privacy Impact

Invite may carry dial `addr` and display name when exported; no network send in this RFC.

## 12. Policy Impact

None.

## 13. Failure Semantics

Invalid document fails schema validation. Missing required fields MUST fail.

## 14. Rollback Plan

Delete schema, fixtures, RFC, unit test.

## 15. Conformance Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures` must pass. Invalid missing-`identity_ref` MUST fail. Valid fixture MUST include `identity_ref` and 64-char `public_key_hex`.

## 16. Migration Plan

None. New optional Desktop invite document.

## 17. Alternatives Considered

- Reuse identity-descriptor full object — rejected; invite is a minimal peer handoff, not a full descriptor ceremony.
- Require `addr` always — rejected; trust-only invite is valid (addr optional per §4a).
- Embed x25519 static — rejected for 0.1; hello binds x25519 on dial (see peer-link).

## 18. Evidence

- phase-e §4a atom `#80`
- desktop-ux §3 P1 onboarding
