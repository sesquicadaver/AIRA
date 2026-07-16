# AIRA demos

Local demos for the MVP operational flow (`aira-flow` / CLI).

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

Library equivalent: `OperationalPlane::submit_problem("Calculate 2 + 2")`.

## Ready Solution reuse

When a Ready Solution Artifact is registered with Reduction-basic, the plane prefers reuse and skips execution capsules.

Evidence in tests: `ready_solution_reuse_skips_execution` in `aira-flow` — `CapsuleCompleted` is absent; payload may carry `reuse:ready_solution`.

## Failure-to-evidence

A missing capsule artifact on `CapsuleCreated` must not invent a Verified Result.

Expected events:

- `CapsuleFailed`
- `FailureEvidenceCreated`
- no successful `VerificationCompleted` for that failure path

CLI path: inject via library / tests (`failure_to_evidence_demo`). Operational CLI focuses on happy-path submit; failure path is covered by conformance C1 and alpha acceptance.
