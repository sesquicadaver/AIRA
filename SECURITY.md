# Security Policy

**Posture:** AIRA reference implementation + **Developer Preview** Desktop — not a production distributed security boundary. See [`docs/implementation-status.md`](docs/implementation-status.md) and [`docs/desktop-ux.md`](docs/desktop-ux.md).

## Reporting

Report vulnerabilities **privately** via [GitHub Security Advisories](https://github.com/sesquicadaver/AIRA/security/advisories) (preferred) or direct contact with repository maintainers. Do not open public issues for exploitable findings until coordinated disclosure.

## Supported versions

| Track | Branch | Notes |
|-------|--------|-------|
| Reference / Developer Preview | `main` | Active development; CI gate on `main` (see below) |
| Releases | tagged alpha | See `docs/RELEASE_NOTES-*.md` |

## Reference security controls (implemented)

### Core & artifacts

- **Canonical signatures** on Object, Event, Artifact, and CSU manifest descriptors (QUEUE `#39`–`#44`); runtime rejects unsigned / tampered descriptors.
- **Immutable object store** — in-place mutation fails closed ([`aira-core`](crates/aira-core)).
- **Private artifacts** denied by default (private artifacts); explicit grant required ([`aira-artifact`](crates/aira-artifact)).
- **Policy gate** + invariant checker in C0/C1 paths ([`aira-policy`](crates/aira-policy), [`aira-conformance`](crates/aira-conformance)).

### CSU isolation

- CSU crates do **not** call each other directly; `scripts/dep_firewall.py` enforces `aira-core` ↛ node/peer/CSU and CSU ↛ CSU (QUEUE `#45`).
- Model-layer CSUs are **local-only** (no network acquisition without explicit policy ALLOW; default DENY).
- In-process CSU sandbox: no peer/Core memory access; failure events on isolation violations — see `run_security_baseline` in [`docs/conformance.md`](docs/conformance.md).

### Local HTTP (`aira-node`)

- Default **loopback** bind; public bind requires explicit `--allow-public-bind` (Analyze-69).
- Optional TLS/mTLS, Bearer token, and multi-tenant CSU authz — [`docs/local-node.md`](docs/local-node.md), [`docs/crypto.md`](docs/crypto.md).
- Mutating routes require Desktop/operator auth contract (Phase E §2.4).

### Peer / post-MVP network

- Peer listen defaults **loopback**; Noise + trust store admission; no auto-trust strangers ([`docs/peer-link.md`](docs/peer-link.md)).
- DHT/relay/gossip/STUN/discv are **prototypes**, not production mesh security.
- Desktop network profiles P0–P6: fail-closed rules in [`docs/desktop-network-profiles.md`](docs/desktop-network-profiles.md).

### Desktop

- Rust-only GUI; no hidden public STUN default; no auto-trust from discovery ([`docs/desktop-gui.md`](docs/desktop-gui.md)).
- Packaging docs: Linux/macOS/Windows Developer Preview tarballs/zips — not codesigned store builds.

### Events & secrets

- Events must not embed secrets; conformance security baseline scans for leakage patterns.

## CI verification

Required check `fmt-clippy-test-schema-c0-c1` ([`docs/ci-governance.md`](docs/ci-governance.md)):

- `dependency firewall`, `cargo test --workspace`
- `schema validate --fixtures`
- conformance **C0** + **C1** + **C2** (job `conformance-c2`; QUEUE `#117`)
- Rust **1.94.0** pinned ([`rust-toolchain.toml`](rust-toolchain.toml))

See [`docs/ci-governance.md`](docs/ci-governance.md) for required checks on `main`.

## Out of scope (not security guarantees today)

```text
production distributed runtime / settlement / CRP
production STUN/discv5 mesh / marketplace
codesign / App Store / MSI enterprise deployment
epistemic CSU / PHM / Book V research tracks
```

## Related

- Implementation matrix: [`docs/implementation-status.md`](docs/implementation-status.md)
- CSU signing: [`docs/csu-development.md`](docs/csu-development.md)
- RFC: [`specs/rfc/AIRA-RFC-0059-security-md-refresh.md`](specs/rfc/AIRA-RFC-0059-security-md-refresh.md)
