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
**Implementation status:** [`docs/implementation-status.md`](docs/implementation-status.md) — **Reference v0.2** (Book 0–IV / schemas / C0–C3 scaffold / CSU basic + epistemic-basic).
**Documentation index:** [`docs/README.md`](docs/README.md).
**Черга задач:** [`QUEUE.md`](QUEUE.md) — Phase C–E **DONE**. Phase F stabilization **DONE** (`#107`–`#119`, [`docs/phase-f-plan.md`](docs/phase-f-plan.md)). Phase G Reference v0.2 **`#120`–`#151` DONE** — [`docs/phase-g-plan.md`](docs/phase-g-plan.md); consolidating RFC [`AIRA-RFC-0069`](specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md). Phase H Protocol depth v0.3 — [`docs/phase-h-plan.md`](docs/phase-h-plan.md) (`#152`–`#183`, лінійно); `#152`–`#157` **DONE**, **`#158` OPEN**. Анти-місія §«What AIRA is not» **ніколи** в QUEUE.

Активний атом: `#158` (Sqlite object path). Далі: durable stores → C3 depth → CRP → settlement receipts → research promotion → RFC-0077 / Reference v0.3. Status: [`docs/implementation-status.md`](docs/implementation-status.md) (**v0.2**).


See `docs/RELEASE_NOTES-v0.1.0-alpha.1.md`, `scripts/prepare-alpha.sh`, `scripts/package-desktop-linux.sh`, `scripts/package-desktop-macos.sh`, `scripts/package-desktop-windows.sh`, plus production-path helpers `scripts/macos-codesign-notarize.sh`, `scripts/package-desktop-windows-msi.sh`, `scripts/package-desktop-linux-deb.sh`.

Long-running daemons: [`docs/runbook-systemd.md`](docs/runbook-systemd.md).

CI merge policy: [`docs/ci-governance.md`](docs/ci-governance.md).

## Build

Rust **1.94.0** (pinned in [`rust-toolchain.toml`](rust-toolchain.toml); `rustup` auto-selects via directory override).

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
