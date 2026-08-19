# OperationalPlane — C1 reference-local flow

**Status (Analyze-86 / QUEUE #51):** `aira_flow::OperationalPlane` is the in-process **C1 reference/demo** pipeline used by local CLI/HTTP submit, `aira-flow` tests, and conformance C0/C1.

It is **not**:

- a production event runtime
- a distributed runtime
- a scheduler
- a federation runtime

Canonical wording (EVO-2 §3.3):

```text
OperationalPlane = локальна референсна демонстрація C1
не production event runtime
не distributed runtime
не scheduler
не federation runtime
```

## What it is

- In-process drain of basic CSUs (context → reduction → execution → verification → evidence).
- Memory object store + memory event log; artifact CAS on disk under the node/CAS root.
- Safety bound on drain iterations (demo fail-closed), not a job scheduler.
- C1 profile (`config.node.profile = "C1"`) for local MVP / alpha acceptance.

Peer/federation/DHT live in `aira-peer` / protocol crates and are **out of this plane**. HTTP (`aira-node`) is a local JSON surface over `LocalSession`, which still uses this same reference plane.

## Pointers

| Surface | Path |
|---------|------|
| Module | `crates/aira-flow/src/plane.rs` |
| Session wrap | `crates/aira-flow/src/local.rs` |
| C1 cases | `crates/aira-conformance/src/c1.rs` |
| C0 causality | `crates/aira-conformance/src/c0.rs` |
| Demos | [demo.md](demo.md) |
| Local node | [local-node.md](local-node.md) |
| Status map | [implementation-status.md](implementation-status.md) |
