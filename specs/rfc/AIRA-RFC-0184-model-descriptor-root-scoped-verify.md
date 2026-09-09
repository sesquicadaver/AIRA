# AIRA-RFC-0184 — Model descriptor root-scoped verify (RFC-D)

## 1. Summary

Phase S `#297`: `ActivatedPointerGate` binds the root-scoped verification keyring (`Keyring::load_node_identity`) for `CasArtifactStore::open` + `resolve`, so node-signed `ArtifactDescriptor` (+ sidecar) and payload evidence both verify after cold reopen without priming the process-global keyring. RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

`#278` / RFC-0167 made **payload** activate-evidence verify root-scoped. Store open/resolve still admitted descriptors via process/thread crypto. Production activates sign both descriptor and payload with the node identity; Desktop reopen does not call `register_node_identity`, so descriptor admit failed before payload verify.

## 3. Motivation

`phase-s-plan` `#297` / post-R audit: descriptor + payload must verify after reopen without process keyring priming.

## 4. Scope

- `bind_thread_crypto(verification_crypto(root))` before `CasArtifactStore::open` in `verify_pointer_ready`
- Fixture signs **both** descriptor and payload with disk identity (no process register)
- Reopen child-process test without priming
- QUEUE → `#298`

## 5. Non-Goals

```text
Identity incomplete-pair (#298)
Observe off-UI hash (#303)
aira-artifact API redesign (resolve_with_keyring)
```

## 6. Compatibility / Security

Local-test-signed fixture descriptors still work (`load_node_identity` rings include local-test). Admission crypto is not weakened — keys come from the install root.

## 7. Rollout

QUEUE `#297` → Analyze-333 → PR; next `#298` Identity incomplete-pair fail-closed.
