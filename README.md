# AIRA

AIRA is a minimal, event-native, artifact-based, CSU-driven system for transforming a **Problem Statement** into a **Verified Result Artifact** with minimal necessary resources.

```text
Problem Statement
  → Context
  → Reuse / Reduction
  → Execution only if needed
  → Verification
  → Verified Result Artifact
  → Evidence
```

## What AIRA is not

- AIRA is **not** a GPU marketplace.
- AIRA is **not** an LLM runtime.
- AIRA is **not** a blockchain network.
- AIRA is **not** a scheduler.
- AIRA Core does **not** solve tasks.
- AIRA Core enables compatible CSU to solve tasks through stable contracts.

## Repository layout

| Path | Role |
|------|------|
| `Manifesto etc/` | Immutable canonical Manifesto sources (Books 0–V + governance) |
| `Meditation_About/` | Immutable historical drafts |
| `Analyze-*/` | Analytical layer (does not change originals) |
| `specs/` | Working copies of Books + governance for implementation |
| `crates/` | Rust MVP workspace |
| `csu/` | Basic CSU crate skeletons |

## Status

**Epic 0–4:** workspace + schemas + C0 Object Store + **Artifact / Event / Policy** (`aira-artifact`, `aira-event`, `aira-policy`, `InvariantChecker`).

CSU runtime is **not** complete yet (Issue Set #35+).

## Build

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p aira-cli -- schema validate --fixtures fixtures
```

```bash
cargo run -p aira-cli -- status
cargo run -p aira-cli -- schema list
```

## License

Apache-2.0 OR MIT.
