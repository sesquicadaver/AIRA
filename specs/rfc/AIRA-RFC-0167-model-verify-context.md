# AIRA-RFC-0167 — Model verify context (RFC-D)

## 1. Summary

Phase Q `#278`: activate evidence verification for observe/admit uses a **root-scoped** keyring (`Keyring::load_node_identity`), not `verify_ed25519` on the process-global ring. Desktop reopen with on-disk identity + activated pointer shows `ready` without test keyring priming. RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

`verify_activate_evidence` called process-global `verify_ed25519`. After Desktop restart, `ensure_local_identity` early-returns without `register_node_identity`, so process ring stays `local-test` while evidence is signed by `aira:identity:desktop.*` → observe `ready: false` despite valid on-disk activation.

## 3. Motivation

`phase-q-plan` `#278` / acceptance: reopen Desktop with activated model must show ready without test-only keyring priming.

## 4. Scope

- `verification_keyring(aira_root)` → `Keyring::load_node_identity` or `with_local_test`
- `ring.verify` instead of `verify_ed25519` in activate evidence path
- In-process + separate-process reopen tests
- QUEUE → `#279`

## 5. Non-Goals

```text
Reachability observation independence (#279)
Changing how activate evidence is signed at acquisition time
Silent migration of identity IDs
```

## 6. Compatibility / Security

Fixtures without `identity/` still verify via local-test in the root-loaded ring. Admission uses the same root-scoped path (not weakened). Process keyring is not mutated by observe.

## 7. Rollout

QUEUE `#278` → Analyze-313 → PR; next `#279` Reachability observation independence.
