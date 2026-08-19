# Analyze-49 — Coordinated local.x25519 rotate

**Scope:** On `aira identity rotate`, also rotate `identity/local.x25519`; `--backup` writes `local.x25519.prev` (+ archive prior). No remote dual-key TrustStore. No Manifesto/Meditation.

**Status:** CLOSED (APPROVE/CLEAR)

## RALPLAN-DR

### Principles
1. Noise static rotates with Ed25519 ceremony (same operator action)
2. Opt-in backup only (`--backup`); mirror restrictive `0600`
3. Keep rotate API in `aira-peer` (no object↔peer cycle)
4. Ed25519 cutover first; then x25519
5. Dual-key remote TrustStore stays Out

### Options
| Option | Verdict |
|--------|---------|
| **A. rotate_noise_static + CLI glue** | Chosen |
| B. Dual-key remote TrustStore | Follow-up (#16) |
| C. Both | Too wide |

### Architect / Critic
**APPROVE** — local-only, testable, no TrustStore schema change.

### Acceptance
- [x] `rotate_noise_static(root, backup)` in aira-peer
- [x] `identity rotate` always rotates x25519; prints new pub (+ backup path)
- [x] `--backup` → `local.x25519.prev`; prior `.prev` archived with stamp
- [x] Tests + docs + QUEUE; APPROVE/CLEAR

### Delivered
- `NoiseStaticRotate` / `rotate_noise_static` / `NODE_X25519_BACKUP_FILE`
- CLI prints `x25519_*` after Ed25519 rotate
- Docs + QUEUE #14 DONE for x25519 micro

### Out
Remote same-id TrustStore dual-key; mTLS; auto peer notify for x25519.
