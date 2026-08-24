# Conformance guide

Repo-level ТЗ → module → tests map: [implementation-status.md](implementation-status.md). Merge gate: [ci-governance.md](ci-governance.md).

## Profiles

| Profile | Runner | Focus | CI on `main` |
|---------|--------|-------|----------------|
| C0 | `run_c0` | ontology, object/artifact immutability, event causality, policy gate, CSU dispatch, acquisition fail-closed | **yes** (job `check`) |
| C1 | `run_c1` | pipeline 2+2, CSU manifests, verified result completeness, failure-to-evidence (**reference** `OperationalPlane`, [operational-plane.md](operational-plane.md)) | **yes** |
| C2 | `run_c2` | partial **local** protocols (M13): envelope/response/identity schemas, discovery Capability≠Node, UNSUPPORTED_VERSION without side effects | **yes** (job `conformance-c2`, QUEUE #117) |
| C3–C5 | — | federated / settlement / research | no (C3 scaffold planned Phase G `#136`) |

Reports validate against `aira:schema:conformance:report:0.1` and are published as immutable `ConformanceArtifact`.

Phase G `#122`–`#124` adds named local C2 cases (idempotency, hash mismatch, unsigned envelope). See [phase-g-plan.md](phase-g-plan.md).

## CLI

```bash
cargo run -p aira-cli -- conformance run --profile C0 --out /tmp/aira-c0
cargo run -p aira-cli -- conformance run --profile C1 --out /tmp/aira-c1
cargo run -p aira-cli -- conformance run --profile C2 --out /tmp/aira-c2
```

## Library

```rust
use aira_conformance::{run_c0, run_c1, run_c2, run_security_baseline, run_alpha_acceptance};

let c0 = run_c0("/tmp/c0")?;
let c1 = run_c1("/tmp/c1")?;
let c2 = run_c2("/tmp/c2")?;
let sec = run_security_baseline("/tmp/sec")?;
let alpha = run_alpha_acceptance("/tmp/alpha")?;
```

## Security baseline (#78)

- Unsigned CSU registration rejected
- Unsigned artifact publish rejected
- Private artifacts (`aira:policy:private`) denied on default resolve
- Obvious secret material rejected in event `payload_ref`

## Alpha acceptance (#80)

`run_alpha_acceptance` checks init layout, Calculate 2+2, failure evidence, and C0/C1 pass.

## Partial C2 (Analyze-46 / Roadmap M13)

Local-only protocol compatibility gate. Wire network C2 / production federation / DHT remain **post-MVP** and out of C0/C1/C2 CI contract.
