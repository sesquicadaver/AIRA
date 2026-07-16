# CSU development guide

## What a CSU is

A Computational Service Unit implements a narrow contract: subscribe to events, optionally read/write artifacts, emit further events. Core does **not** solve tasks; CSUs do.

## Manifest

Signed `CsuManifest` (`aira:schema:csu:manifest:0.1`):

- `csu_id`, `csu_type`, `abi_version` (`0.1`)
- `event_subscriptions` / `event_outputs`
- `sandbox` with `network: none`, `secret_access: none` for basic CSUs
- non-empty `signature.signature_value` (unsigned manifests are rejected)

Validate:

```bash
cargo run -p aira-cli -- schema validate \
  --schema aira:schema:csu:manifest:0.1 \
  --file path/to/manifest.json
```

Register locally:

```bash
cargo run -p aira-cli -- --root /tmp/aira-demo csu register --manifest path/to/manifest.json --activate
cargo run -p aira-cli -- --root /tmp/aira-demo csu list
```

## In-process trait

Implement `aira_csu::Csu`:

- `manifest(&self) -> &CsuManifest`
- `on_event(&mut self, event, ctx) -> Result<CsuOutput, _>`

Use `aira_csu::support::{basic_manifest, make_event, make_artifact}` for MVP helpers.

Basic set under `csu/`:

| Crate | Role |
|-------|------|
| context-basic | ProblemSubmitted → Context |
| reduction-basic | reuse or CapsuleCreated |
| execution-basic | safe math/text actions |
| verification-basic | Verified Result |
| evidence-basic | failure/result evidence |
| artifact-basic | publish/resolve/supersede ops |

## Isolation baseline

Default sandbox: no filesystem/network/device/secret access; in-process only. Do not emit secret material into `payload_ref` (event log rejects obvious secret patterns).
