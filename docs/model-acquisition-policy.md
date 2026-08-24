# Model acquisition policy (fail-closed)

Phase D/F reference rules for `csu/model-acquisition` (`aira:csu:model.acquisition`). **Not** a marketplace; local policy gate only ([`phase-d-plan.md`](phase-d-plan.md) D4).

## Policy file

Optional: `.aira/models/acquisition.policy.json` (`aira:schema:model:acquisition-policy:0.1`).

| Field | Default when absent | Effect |
|-------|---------------------|--------|
| `auto_download` | — (no file) | Download gate **DENY** |
| `share_custom_models` | — (no file) | Publish/share gate **DENY** |

## Gates (no implicit ALLOW)

| Operation | API | Without ALLOW |
|-----------|-----|----------------|
| Download request | `request_download` | DENY + decision artifact + `PolicyEvaluated`-style event payload |
| Publish request | `request_publish` | DENY + share decision pointer + event |
| Quarantine fetch | `fetch_to_quarantine` | `FetchOutcome::Denied` — no bytes copied |
| Local publish | `publish_local` | `PublishOutcome::Denied` — no ShareOffer / capability ad |

ALLOW requires explicit policy flags (`auto_download=true` or `share_custom_models=true`). Gate evaluation alone never transfers weights or writes ShareOffer bytes.

## Tests

- `cargo test -p aira-csu-model-acquisition`
- C0: `c0.acquisition.fail_closed`
- Regression: `fail_closed_audit_download_and_publish_without_allow`

See also [`SECURITY.md`](../SECURITY.md) (model-layer local-only, default DENY).
