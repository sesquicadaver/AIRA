# CSU development guide

See also: [model-acquisition-policy.md](model-acquisition-policy.md) (local model layer); [`csu-sdk.md`](csu-sdk.md) (third-party CSU contract; **not** a CSU marketplace; QUEUE `#145`).

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

## Basic set under `csu/`

| Crate | Role |
|-------|------|
| context-basic | ProblemSubmitted → Context |
| reduction-basic | reuse or CapsuleCreated |
| execution-basic | safe math/text actions |
| verification-basic | Verified Result |
| evidence-basic | failure/result evidence |
| epistemic-basic | EPI-001 assessment (Evidence/Confidence/Scope/Status; `#146`) |
| artifact-basic | publish/resolve/supersede ops |
| model-acquisition | local model download/publish policy gate (not marketplace) |
| model-rating | contextual rating evidence (local-only) |
| model-recommendation | advisory upgrade recommendation (local-only) |

**Phase G `#147`:** epistemic assessment path (plane/CLI roundtrip).

## Isolation baseline

Default sandbox: no filesystem/network/device/secret access; in-process only. Do not emit secret material into `payload_ref` (event log rejects obvious secret patterns).

`scripts/dep_firewall.py` enforces `aira-core` ↛ node/peer/CSU and CSU ↛ CSU (QUEUE `#45`).

CSU `on_event` is invoked by the local C1 **reference/demo** `OperationalPlane` ([operational-plane.md](operational-plane.md)), not a production event, scheduler, or federation runtime.

## Anti-mission

AIRA is **not** a GPU marketplace, LLM runtime host, blockchain network, or scheduler. Model/GPU references belong in **Artifacts** and model-layer CSUs, not Core ontology. README §«What AIRA is not»; [canonical-terminology.md](canonical-terminology.md).
