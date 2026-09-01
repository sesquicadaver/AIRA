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
| `analysis/` | Analytical layer — `Analyze-N/` cycles (does not change originals) |
| `specs/` | Working copies of Books + governance for implementation |
| `crates/` | Rust MVP workspace |
| `csu/` | Basic CSU crate skeletons |

## Status

**Epic 0–11:** workspace through Partial Local C2 + **MVP Alpha**.
**Post-MVP:** M11 HTTP; Alpha.2 Ed25519; Identity Keyring; Plane node signing; Trust; Peer P0–P6 + DHT/relay/gossip ([`docs/crypto.md`](docs/crypto.md), [`docs/peer-link.md`](docs/peer-link.md), [`docs/local-node.md`](docs/local-node.md), [`docs/desktop-network-profiles.md`](docs/desktop-network-profiles.md)).
**Desktop:** Developer Preview Linux/macOS/Windows — [`docs/desktop-ux.md`](docs/desktop-ux.md); network profiles P0–P6 **DONE** (Phase E); packaging helpers `#148`–`#150`.
**OperationalPlane:** C1 reference/demo only — [`docs/operational-plane.md`](docs/operational-plane.md) (not production event/scheduler/federation runtime).
**Implementation status:** [`docs/implementation-status.md`](docs/implementation-status.md) — **Reference v0.3-strict** (local Book-gap remainder after v0.3-stable; [`AIRA-RFC-0077`](specs/rfc/AIRA-RFC-0077-phase-h-protocol-depth-v0.3.md); [`AIRA-RFC-0078`](specs/rfc/AIRA-RFC-0078-phase-i-semantic-stabilization.md); [`AIRA-RFC-0096`](specs/rfc/AIRA-RFC-0096-phase-j-book-gap-local.md)).
**Documentation index:** [`docs/README.md`](docs/README.md).
**Черга задач:** [`QUEUE.md`](QUEUE.md) — Phase C–E **DONE**. Phase F stabilization **DONE** (`#107`–`#119`, [`docs/phase-f-plan.md`](docs/phase-f-plan.md)). Phase G Reference v0.2 **`#120`–`#151` DONE** — [`docs/phase-g-plan.md`](docs/phase-g-plan.md); consolidating RFC [`AIRA-RFC-0069`](specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md). Phase H Protocol depth v0.3 — [`docs/phase-h-plan.md`](docs/phase-h-plan.md) (`#152`–`#183` **DONE** @ RFC-0077). Phase I semantic contracts — [`docs/phase-i-plan.md`](docs/phase-i-plan.md) (`#184` `#185` `#186` `#187` `#188` `#189` `#190` `#191` `#192` `#193` `#194` `#195` `#196` `#197` `#198` **DONE** @ RFC-0078). Phase J Book-gap local remainder — [`docs/phase-j-plan.md`](docs/phase-j-plan.md) (`#199` `#200` `#201` `#202` `#203` `#204` `#205` `#206` `#207` `#208` **DONE** @ RFC-0096). Phase K Local LLM Execution CSU — [`docs/phase-k-plan.md`](docs/phase-k-plan.md) (`#209` `#210` `#211` **DONE**; `#212` `#213` `#214` `#215` `#216` OPEN). Анти-місія §«What AIRA is not» **ніколи** в QUEUE.

Phase J [`docs/phase-j-plan.md`](docs/phase-j-plan.md) **DONE**. Phase K [`docs/phase-k-plan.md`](docs/phase-k-plan.md) **IN PROGRESS** (`#211` execution-llm mock DONE; first OPEN `#212`). Status: [`docs/implementation-status.md`](docs/implementation-status.md) (**v0.3-strict**). RFC-P process: [`docs/rfc-p-promotion.md`](docs/rfc-p-promotion.md). QUEUE J closed.


See `docs/RELEASE_NOTES-v0.1.0-alpha.1.md`, `scripts/prepare-alpha.sh`, `scripts/package-desktop-linux.sh`, `scripts/package-desktop-macos.sh`, `scripts/package-desktop-windows.sh`, plus production-path helpers `scripts/macos-codesign-notarize.sh`, `scripts/package-desktop-windows-msi.sh`, `scripts/package-desktop-linux-deb.sh`.

Long-running daemons: [`docs/runbook-systemd.md`](docs/runbook-systemd.md).

CI merge policy: [`docs/ci-governance.md`](docs/ci-governance.md).

## Build

Rust **1.94.0** (pinned in [`rust-toolchain.toml`](rust-toolchain.toml); Cargo `rust-version = "1.94"`; `rustup` auto-selects via directory override).

```bash
python3 scripts/dep_firewall.py --self-test
python3 scripts/dep_firewall.py
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
cargo run -p aira-cli -- conformance run --profile C2 --out /tmp/aira-c2
```

```bash
cargo run -p aira-cli -- status
cargo run -p aira-cli -- schema list
```

## License

Apache-2.0 OR MIT.

## Support the project

If this project is useful to you, you may support its development with a voluntary donation in USDT.

Donations are optional and do not provide ownership, equity, tokens, governance rights, paid support, priority service, or any investment return.

### USDT donations

| Network | Address |
|---|---|
| USDT ERC-20 / Ethereum | 0xfa9821efd142228d53e1418fe335bb1cd8ff3c39 |
| USDT TRC-20 / Tron | TNnhueeGqujf6AAUhcgissoEkL7tdzmqQv |

### Important

Please make sure that the selected network matches the address type.

- Send **USDT ERC-20** only to the Ethereum address.
- Send **USDT TRC-20** only to the Tron address.

Transactions sent to the wrong network may be permanently lost.

Thank you for supporting the project.
