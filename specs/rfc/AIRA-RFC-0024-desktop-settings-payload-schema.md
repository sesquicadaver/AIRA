# AIRA-RFC-0024 — Desktop settings schema

## 1. Summary

Additive JSON Schema `aira:schema:desktop:settings:0.1` describes the AIRA Desktop settings document (OS config path). It is **not** a Core entity and does not add a canonical `ArtifactType`. E1 uses `network_profile=P0` only.

## 2. Problem Statement

Phase E (`#75`) needs a fixed settings contract so `#76` lifecycle and `#78` tray share the same fields (`open_ui_on_start`, `autostart_on_login`, fixed `http_listen`, `instance_id`, auth-mode placeholders) without inventing informal JSON.

## 3. Motivation

[`docs/phase-e-plan.md`](../../docs/phase-e-plan.md) §2–§3 and [`docs/desktop-ux.md`](../../docs/desktop-ux.md) require schema-first Desktop settings before the orchestrator.

## 4. Scope

- `schemas/desktop/settings.schema.json`
- Valid/invalid fixtures in `fixtures/manifest.json`
- Unit coverage in `aira-schema`

## 5. Non-Goals

```text
aira desktop start|stop|status (#76)
token generation / Desktop IPC runtime (#76)
tray/GUI (#78)
OS autostart hooks (#78)
peer profiles > P0 / P1 onboarding
Book 0 / aira-core / C1 change
```

## 6. Current Behavior

No `aira:schema:desktop:*` ids. Desktop settings path is documented only.

## 7. Proposed Change

Required fields:

```text
payload_schema = aira:schema:desktop:settings:0.1
network_profile ∈ {P0…P6}
open_ui_on_start (boolean)
autostart_on_login (boolean)
http_listen (string host:port; no auto-increment semantics in schema)
instance_id (string)
http_auth_mode ∈ {bearer_token, desktop_ipc}
```

Optional: `http_token_ref` (path/ref placeholder, not the secret), `peer_listen` (string|null). `additionalProperties: false`. Missing `instance_id` MUST fail.

## 8. Affected Books / Schemas / Tests

- Schema Pack: `schemas/desktop/settings.schema.json` (additive)
- Fixtures: `fixtures/valid/desktop/settings.json`, `fixtures/invalid/desktop/settings-missing-instance-id.json`
- Tests: `aira-schema` + `schema validate --fixtures`
- Books 0–III: none

## 9. Compatibility Impact

Additive. Existing fixtures and C0/C1 unchanged.

## 10. Security Impact

Schema only. Does not store bearer secrets; `http_token_ref` is a locator placeholder. Runtime auth is `#76`.

## 11. Privacy Impact

`instance_id` is a local install identifier in fixtures/docs; no network export in this RFC.

## 12. Policy Impact

None beyond documenting default Desktop posture (P0, autostart off) as data shape.

## 13. Failure Semantics

Invalid document fails `schema validate --fixtures`. Missing required fields MUST fail.

## 14. Rollback Plan

Delete schema, fixtures, this RFC, and the unit test; registry walkdir stops loading it.

## 15. Conformance Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures` must pass. Invalid missing-`instance_id` MUST fail. Valid fixture MUST have `network_profile: "P0"`.

## 16. Migration Plan

None. New optional settings document for Desktop.

## 17. Alternatives Considered

- Reuse model acquisition-policy shape with signatures — rejected; Desktop settings are OS config, not CustomArtifact evidence.
- Const-only `network_profile: P0` — rejected; enum reserves P1–P6 for later addenda without schema `$id` churn.
- Embedding raw HTTP token in settings JSON — rejected; only `http_token_ref` placeholder.

## 18. Evidence

- [`docs/phase-e-plan.md`](../../docs/phase-e-plan.md) atom `#75`
- [`docs/desktop-ux.md`](../../docs/desktop-ux.md) §6
- [`NEXT_PROBLEM.md`](../../NEXT_PROBLEM.md) RESOLVED

## 19. Open Questions

Exact filesystem path mapping for `http_token_ref` on Linux/macOS/Windows — `#76`.
