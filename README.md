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

**Epic 0–11:** workspace through Partial Local C2 + **MVP Alpha**.
**Post-MVP:** M11 HTTP; Alpha.2 Ed25519; Identity Keyring; Plane node signing; Trust; Peer P0+CLI (`docs/crypto.md`, `docs/peer-link.md`).
**Черга задач:** [`QUEUE.md`](QUEUE.md) (наступний slice: Analyze-38 Peer pubkey notify).

See `docs/RELEASE_NOTES-v0.1.0-alpha.1.md` and `scripts/prepare-alpha.sh`.

## Build

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p aira-cli -- schema validate --fixtures fixtures
```

```bash
# Local node smoke
cargo run -p aira-cli -- --root /tmp/aira-demo init
cargo run -p aira-cli -- --root /tmp/aira-demo identity create
cargo run -p aira-cli -- --root /tmp/aira-demo problem submit --text "Calculate 2 + 2"
cargo run -p aira-node -- --root /tmp/aira-demo --text "Calculate 2 + 2"
# Local HTTP API (loopback)
cargo run -p aira-node -- --root /tmp/aira-demo --http --listen 127.0.0.1:8787
cargo run -p aira-cli -- conformance run --profile C0 --out /tmp/aira-c0
cargo run -p aira-cli -- conformance run --profile C1 --out /tmp/aira-c1
```

```bash
cargo run -p aira-cli -- status
cargo run -p aira-cli -- schema list
```

## License

Apache-2.0 OR MIT.
