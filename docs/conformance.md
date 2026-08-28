# Conformance guide

Repo-level ТЗ → module → tests map: [implementation-status.md](implementation-status.md). Merge gate: [ci-governance.md](ci-governance.md).

## Profiles

| Profile | Runner | Focus | CI on `main` |
|---------|--------|-------|----------------|
| C0 | `run_c0` | ontology, object/artifact immutability, event causality, policy gate, CSU dispatch, acquisition fail-closed | **yes** (job `check`) |
| C1 | `run_c1` | pipeline 2+2, CSU manifests, external partner fixture (`c1.csu.external_partner_fixture`), verified result completeness, failure-to-evidence (**reference** `OperationalPlane`, [operational-plane.md](operational-plane.md)) | **yes** |
| C2 | `run_c2` | partial **local** protocols (M13): envelope/response/identity schemas, discovery Capability≠Node, UNSUPPORTED_VERSION without side effects | **yes** (job `conformance-c2`, QUEUE #117) |
| C3 | `run_c3` | **local federation scaffold** (#141); Phase H глибочить cases / capability / CRP (`#160`–`#170`) | **no** — optional only after `#164`; **not** a merge gate ([criteria](ci-governance.md#profile-c3-queue-141-scaffold-153-governance)) |
| C4 | `run_c4` (Phase H `#175`) | settlement audit receipts scaffold | no |
| C5 | `run_c5` (Phase H `#180`) | research separation + promotion gate scaffold | no |

Reports validate against `aira:schema:conformance:report:0.1` and are published as immutable `ConformanceArtifact`.

Phase G `#122`–`#124` adds named local C2 cases (idempotency, hash mismatch, unsigned envelope). SEC-2 (`#135`) adds `c2.protocol.envelope_canonical_mutations` and `c2.protocol.response_canonical_mutations`. See [phase-g-plan.md](phase-g-plan.md).

### C3 governance (#153)

- **Today:** run locally; **no** `conformance-c3` job in `ci.yml`; **not** in branch-protection required checks.
- **Optional CI job (`#164`):** only when `run_c3` has ≥6 stable local cases and the job stays non-required — see [ci-governance.md](ci-governance.md) §Profile C3.
- **Merge gate:** only via a later dedicated QUEUE atom + RFC after sustained green optional job; never silently.
- **`#153` does not** add workflow YAML (that is `#164` only).

## CLI

```bash
cargo run -p aira-cli -- conformance run --profile C0 --out /tmp/aira-c0
cargo run -p aira-cli -- conformance run --profile C1 --out /tmp/aira-c1
cargo run -p aira-cli -- conformance run --profile C2 --out /tmp/aira-c2
cargo run -p aira-cli -- conformance run --profile C3 --out /tmp/aira-c3
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
- **`sec.trust_excludes_local_test`** (SEC-1 / QUEUE #134): runtime `trust.json` excludes `aira:identity:local-test`; upsert refused; legacy entry stripped on `ensure_trust_defaults`
- **`sec.producer_identity_binding`** (SEC-3 / QUEUE #136): Object/Event/Artifact require `signature.key_ref == producer_identity`; stores reject cross-identity descriptors
- **`sec.event_equivocation`** (SEC-4 / QUEUE #137): same `event_id` + different canonical hash → `EventError::Equivocation`; C2 `c2.event.publish_equivocation` → `EQUIVOCATION`
- **`sec.artifact_admission`** (SEC-5 / QUEUE #138): no post-verify descriptor mutation; `content_ref` mismatch rejected; supersession persists on reopen

## Alpha acceptance (#80)

`run_alpha_acceptance` checks init layout, Calculate 2+2, failure evidence, and C0/C1 pass.

## Partial C2 (Analyze-46 / Roadmap M13)

Local-only protocol compatibility gate. Wire network C2 / production federation / DHT remain **post-MVP** and out of C0/C1/C2 CI contract.
