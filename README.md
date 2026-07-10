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

**Epic 0 / Epic 1 snapshot:** Cargo workspace + toolchain + CI + `specs/` copies.

Runtime Object/Event/Artifact/CSU logic is **not** implemented yet (Issue Set #22+).

## Build

```bash
cargo metadata
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CLI skeleton:

```bash
cargo run -p aira-cli -- status
```

## License

Apache-2.0 OR MIT.
