# AIRA demos

Local **C1 reference/demo** flow (`aira-flow` / CLI). [`OperationalPlane`](operational-plane.md) is not a production event runtime, scheduler, or federation runtime.

**Queue (`#387` shared-tip rule):** **перший OPEN `#392`** — [`QUEUE.md`](../QUEUE.md) / [`phase-z-plan.md`](phase-z-plan.md) (`#390`–`#391` DONE). Audit `#376`–`#389` **DONE** (closed). Do not treat closed Phase G/X tips as current.

## Prerequisites

```bash
cargo build -p aira-cli -p aira-node
ROOT=/tmp/aira-demo
cargo run -p aira-cli -- --root "$ROOT" init
cargo run -p aira-cli -- --root "$ROOT" identity create
```

## Calculate 2 + 2 → Verified Result Artifact (legacy OP-001)

**Honesty (`#384` / `#387`):** `Calculate 2 + 2` / OP-001 is **legacy non-normative**. Normative C1 merge smoke is process Executed + VRA schema fixtures — not a Desktop calculator. Product Work is `text.generate.local` (executed, not VERIFIED); see [`desktop-gui.md`](desktop-gui.md).

Historical CLI / library path (still useful for local C1 reference):

```bash
cargo run -p aira-cli -- --root "$ROOT" problem submit --text "Calculate 2 + 2"
```

Expected on that path:

- `status completed`
- JSON `result: 4.0`, `verification_status: "VERIFIED"`
- Event chain includes `ProblemSubmitted` → `ContextResolved` → `CapsuleCreated` → `CapsuleCompleted` → `ResultPublished`

Library equivalent: `OperationalPlane::submit_problem("Calculate 2 + 2")` (C1 reference plane; [operational-plane.md](operational-plane.md)).

Desktop GUI Work tab does **not** promise a calculator returning `4.0` + **VERIFIED**; it posts generate-local work to the supervised node ([`desktop-gui.md`](desktop-gui.md)).

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
