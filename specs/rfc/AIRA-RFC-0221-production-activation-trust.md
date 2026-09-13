# AIRA-RFC-0221 — Production activation trust (RFC-D)

## 1. Summary

QUEUE `#337` (Phase W / Pack 1 residual honesty): production activation **MUST NOT** fall back to implicit `aira:identity:local-test`. Node identity is required; local-test evidence issuers are rejected. Explicit fixture trust is opt-in via `models/activation.trust.fixture` (written only by test helpers such as `ActivatedPointerGate::install_fixture`).

## 2. Problem Statement

`ActivatedPointerGate::verification_crypto` fell back to `Keyring::with_local_test()` on any identity load error. `Keyring::load_node_identity` also embedded local-test, so production admit could treat test-key evidence as valid (audit A6 / S16).

## 3. Motivation

Phase W [`docs/phase-w-plan.md`](../../docs/phase-w-plan.md); audit `d1115f2` A6; parent RFC-0215 reserved until `#342`.

## 4. Scope

- `ActivationTrustMode::{Production,Fixture}` + fixture marker
- Production: require node identity; `Keyring::without_local_test()`; reject local-test issuer
- Fixture: `install_fixture` writes marker; local-test evidence allowed
- `activate_verified` / verify-evidence publish: root-scoped node signing; same production trust on verify
- `AcquisitionError::ActivateProductionTrust`

## 5. Non-Goals

```text
Safe weights materialization (#338)
Backend verified binding (#339)
Rewriting SEC-1 runtime TrustStore
Removing local-test from process keyring globally
```

## 6. Compatibility

Honest Desktop/CLI roots with node identity unchanged. Test fixtures must use `install_fixture` (marker) or a real node identity.

## 7. Acceptance

```text
No identity / broken identity → production deny (no local-test fallback)
local-test evidence issuer → production deny
install_fixture marker → fixture admit OK
Node-signed production path → admit OK
Tip → first OPEN #338
```

## 8. References

- QUEUE `#337` · Analyze-374
- Parent: RFC-0215; prior RFC-0220; Phase L RFC-0112; Phase Q RFC-0184
