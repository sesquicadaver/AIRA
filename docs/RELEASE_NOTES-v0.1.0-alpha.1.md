# AIRA MVP v0.1.0-alpha.1 — Release Notes

**Date:** 2026-07-16  
**Tag intent:** `aira-mvp-v0.1.0-alpha.1`

## Highlights

- Local operational flow: Problem Statement → Verified Result Artifact
- Basic CSU set (context/reduction/execution/verification/evidence/artifact)
- CLI + `aira-node` local plane
- C0/C1 conformance runners + security baseline + alpha DoD suite
- Partial local C2: Protocol Envelope, AIRA-EP/AP, Identity Descriptor, Discovery

## Artifacts

| Item | Location |
|------|----------|
| `aira` CLI | `cargo build -p aira-cli --release` → `target/release/aira` |
| `aira-node` | `cargo build -p aira-node --release` → `target/release/aira-node` |
| Schema pack | `schemas/` |
| Fixtures | `fixtures/` |
| Docs | `docs/demo.md`, `docs/csu-development.md`, `docs/local-node.md`, `docs/conformance.md` |
| Prepare script | `scripts/prepare-alpha.sh` |

## Generate conformance reports

```bash
bash scripts/prepare-alpha.sh
# reports under release/alpha/conformance/
```

## Out of scope for alpha

- Real cryptographic verification beyond structural TESTSIG
- Network federation / settlement protocols
- YAML config parity
- ML/GPU/blockchain dependencies (intentionally absent)
