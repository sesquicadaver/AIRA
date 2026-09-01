# AIRA demos

Local **C1 reference/demo** flow (`aira-flow` / CLI). [`OperationalPlane`](operational-plane.md) is not a production event runtime, scheduler, or federation runtime.

**Queue:** Phase G OPEN `#120`–`#146` — [`phase-g-plan.md`](phase-g-plan.md).

## Prerequisites

```bash
cargo build -p aira-cli -p aira-node
ROOT=/tmp/aira-demo
cargo run -p aira-cli -- --root "$ROOT" init
cargo run -p aira-cli -- --root "$ROOT" identity create
```

## Calculate 2 + 2 → Verified Result Artifact

```bash
cargo run -p aira-cli -- --root "$ROOT" problem submit --text "Calculate 2 + 2"
```

Expected:

- `status completed`
- JSON `result: 4.0`, `verification_status: "VERIFIED"`
- Event chain includes `ProblemSubmitted` → `ContextResolved` → `CapsuleCreated` → `CapsuleCompleted` → `ResultPublished`

Library equivalent: `OperationalPlane::submit_problem("Calculate 2 + 2")` (C1 reference plane; [operational-plane.md](operational-plane.md)).

Desktop GUI: tab **Work** / **Робота** in `aira-desktop` posts the same `POST /v1/problems` to the supervised local node and shows **4.0** + **VERIFIED** first; hashes and signatures stay under Details ([`desktop-gui.md`](desktop-gui.md)).

## Ready Solution reuse

When a Ready Solution Artifact is registered with Reduction-basic, the plane prefers reuse and skips execution capsules.

`LocalSession` also persists `problems/reuse-index.json` (QUEUE `#189` / RFC-0087): a second submit of the same problem text reuses the stored verified artifact without Execution.

Evidence in tests: `ready_solution_reuse_skips_execution` and `plane_reduction_binds_reuse_index_without_enable_ready_solution` (durable `reuse-index.json`, no `enable_ready_solution`) and `local_session_repeat_problem_reuses_without_execution` in `aira-flow` — `CapsuleCompleted` is absent; payload may carry `reuse:ready_solution`.

`verification-basic` (`#205` / RFC-0101) does not VERIFIED a `text.echo` / `text.uppercase` output whose claimed `result` string disagrees with `expression` (output body or capsule).

`OperationalPlane` (`#206` / RFC-0102) rejects `claim_kind: Claim` without `evidence_refs`; `Assumption` / `Hypothesis` may omit evidence.

Completed Calculate 2 + 2 (`#207` / RFC-0103) also writes an epistemic-assessment artifact (`latest_epistemic_assessment`); not a full Epistemic plane.

## Failure-to-evidence

A missing capsule artifact on `CapsuleCreated` must not invent a Verified Result.

Expected events:

- `CapsuleFailed`
- `FailureEvidenceCreated`
- no successful `VerificationCompleted` for that failure path

CLI path: inject via library / tests (`failure_to_evidence_demo`). Operational CLI focuses on happy-path submit; failure path is covered by conformance C1 and alpha acceptance.

## Conformance smoke (C0 / C1 / C2)

```bash
cargo run -p aira-cli -- conformance run --profile C0 --out /tmp/aira-c0
cargo run -p aira-cli -- conformance run --profile C1 --out /tmp/aira-c1
cargo run -p aira-cli -- conformance run --profile C2 --out /tmp/aira-c2
```

C0/C1 are the primary MVP gate; C2 runs in CI job `conformance-c2` ([`ci-governance.md`](ci-governance.md), [`conformance.md`](conformance.md)).
