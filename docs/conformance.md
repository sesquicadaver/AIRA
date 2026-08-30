# Conformance guide

Repo-level ТЗ → module → tests map: [implementation-status.md](implementation-status.md) (**Reference v0.3**). Merge gate: [ci-governance.md](ci-governance.md).

## Reference v0.3 index (`#182`)

| Profile | Role | CI on `main` |
|---------|------|----------------|
| C0 / C1 | Local Core + operational node | **yes** (`fmt-clippy-test-schema-c0-c1`) |
| C2 | Partial local protocols | **yes** (`conformance-c2`) |
| C3 | Local federation + CAP + CRP | **optional** (`conformance-c3`, not a merge gate) |
| C4 | Settlement audit receipts scaffold | no |
| C5 | Research separation + promotion gate scaffold | no |

C4/C5 are local scaffolds only. Research remains **RESEARCH**. Consolidating Phase H RFC: [`AIRA-RFC-0077`](../specs/rfc/AIRA-RFC-0077-phase-h-protocol-depth-v0.3.md). Phase I semantic contracts: [`phase-i-plan.md`](phase-i-plan.md) (`#184`–`#190` **DONE**; first OPEN `#191`).

## Profiles

| Profile | Runner | Focus | CI on `main` |
|---------|--------|-------|----------------|
| C0 | `run_c0` | ontology, object/artifact immutability, event causality, policy gate, CSU dispatch, acquisition fail-closed | **yes** (job `check`) |
| C1 | `run_c1` | pipeline 2+2, CSU manifests, external partner fixture (`c1.csu.external_partner_fixture`), verified result completeness, failure-to-evidence (**reference** `OperationalPlane`, [operational-plane.md](operational-plane.md)) | **yes** |
| C2 | `run_c2` | partial **local** protocols (M13): envelope/response/identity schemas, discovery Capability≠Node, UNSUPPORTED_VERSION without side effects | **yes** (job `conformance-c2`, QUEUE #117) |
| C3 | `run_c3` | **8 named local cases**: federation×4 + CAP + export_deny (#163) + `c3.crp.reject_node_route` (#167) + `c3.crp.route_candidate` (#170) | **optional job** `conformance-c3` (#164) — **not** a merge gate ([governance](ci-governance.md#job-conformance-c3-queue-164)) |
| C4 | `run_c4` (Phase H `#175`) | **3 named local cases**: receipt emit/verify + privacy reject + link prior CRP route (RFC-0081) | no |
| C5 | `run_c5` (Phase H `#180`) | **3 named local cases**: research separation + promotion gate reject + candidate schema (RFC-0083); process [`rfc-p-promotion.md`](rfc-p-promotion.md) | no |

Reports validate against `aira:schema:conformance:report:0.1` and are published as immutable `ConformanceArtifact`.

Phase G `#122`–`#124` adds named local C2 cases (idempotency, hash mismatch, unsigned envelope). SEC-2 (`#135`) adds `c2.protocol.envelope_canonical_mutations` and `c2.protocol.response_canonical_mutations`. See [phase-g-plan.md](phase-g-plan.md).

### C3 governance (#153)

- **Today:** CLI/`run_c3` available; optional workflow job `conformance-c3` (#164); **not** in branch-protection required checks.
- **Optional CI job (`#164` DONE):** workflow job `conformance-c3` runs on PR/`main`; **not** in branch-protection required checks — see [ci-governance.md](ci-governance.md) §Job `conformance-c3`.
- **Merge gate (future):** only after a later QUEUE atom + RFC; never silently add C3 to required checks.
### C3 named cases (`run_c3`, #163 + #167 + #170)

1. `c3.federation.descriptor_verify`
2. `c3.federation.join_membership`
3. `c3.federation.leave_clears`
4. `c3.federation.rejoin_after_leave`
5. `c3.capability.advertisement`
6. `c3.federation.export_deny`
7. `c3.crp.reject_node_route`
8. `c3.crp.route_candidate`

- **Merge gate:** only via a later dedicated QUEUE atom + RFC after sustained green optional job; never silently.
- **`#164` added** the optional `conformance-c3` workflow job; required checks unchanged.

### C4 named cases (`run_c4`, #175)

1. `c4.settlement.receipt_emit_verify`
2. `c4.settlement.privacy_reject`
3. `c4.settlement.link_prior_route`

- **Merge gate:** no — scaffold only (RFC-0081); not a CI job.
- **CLI:** `conformance run --profile C4`

### C5 named cases (`run_c5`, #180)

1. `c5.research.separation`
2. `c5.promotion.gate_reject`
3. `c5.promotion.candidate_schema`

- **Merge gate:** no — scaffold only (RFC-0083); not a CI job.
- **CLI:** `conformance run --profile C5`

## CLI

```bash
cargo run -p aira-cli -- conformance run --profile C0 --out /tmp/aira-c0
cargo run -p aira-cli -- conformance run --profile C1 --out /tmp/aira-c1
cargo run -p aira-cli -- conformance run --profile C2 --out /tmp/aira-c2
cargo run -p aira-cli -- conformance run --profile C3 --out /tmp/aira-c3
cargo run -p aira-cli -- conformance run --profile C4 --out /tmp/aira-c4
cargo run -p aira-cli -- conformance run --profile C5 --out /tmp/aira-c5
```

## Library

```rust
use aira_conformance::{run_c0, run_c1, run_c2, run_c3, run_c4, run_c5, run_security_baseline, run_alpha_acceptance};

let c0 = run_c0("/tmp/c0")?;
let c1 = run_c1("/tmp/c1")?;
let c2 = run_c2("/tmp/c2")?;
let c3 = run_c3("/tmp/c3")?;
let c4 = run_c4("/tmp/c4")?;
let c5 = run_c5("/tmp/c5")?;
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
