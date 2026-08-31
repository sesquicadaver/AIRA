# CI governance (QUEUE #109, #120, #153)

Canonical mapping between GitHub Actions and merge policy for `main`. Workflow source: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

**Contract tests:** `crates/aira-desktop-runtime/tests/ci_governance_doc.rs` (`cargo test -p aira-desktop-runtime ci_governance`); living spec C0 smoke `living_spec_smoke.rs` (`#121`).

## Required status check (merge gate)

| Check name | Job | Blocks merge when |
|------------|-----|-------------------|
| `fmt-clippy-test-schema-c0-c1` | `check` | fmt, clippy, tests, schema fixtures, or C0/C1 conformance fail |
| `conformance-c2` | `conformance-c2` | C2 conformance fails (QUEUE #117) |

Configure on `main` (GitHub **Settings → Branches → Branch protection**):

1. **Require a pull request before merging** (recommended).
2. **Require status checks to pass** → select `fmt-clippy-test-schema-c0-c1` and `conformance-c2`.
3. **Require branches to be up to date before merging** (recommended; matches autopilot rebase policy).

`develop` uses the same workflow on push/PR but branch protection is optional.

## Branch protection checklist (QUEUE #120)

Manual verification on GitHub **Settings → Branches → `main` → Branch protection rule**:

| Required status check name | Workflow job key | Source in `ci.yml` |
|----------------------------|------------------|---------------------|
| `fmt-clippy-test-schema-c0-c1` | `check` | `jobs.check.name` |
| `conformance-c2` | `conformance-c2` | `jobs.conformance-c2.name` |

Checklist:

1. **Require status checks to pass** is enabled.
2. Both check names above appear in the rule (exact strings; GitHub shows job `name:` values).
3. **Require branches to be up to date before merging** (recommended).

RFC: [`AIRA-RFC-0070`](../../specs/rfc/AIRA-RFC-0070-ci-branch-protection-sync.md).

## Workflow triggers

```text
push:    main, develop
pull_request: main, develop
```

## Toolchain

- Rust **1.94.0** (`rust-toolchain.toml` + SHA-pinned `dtolnay/rust-toolchain` with `toolchain: 1.94.0`).
- Workspace Cargo `rust-version = "1.94"` (QUEUE `#197` / RFC-0095) — aligned with CI, not the stale 1.75 claim.
- GitHub Actions `uses:` are pinned to commit SHA (tag in comment). Floating `@v4` / `@v2` tags are not used.
- `RUSTFLAGS=-D warnings` for all workspace builds in CI.

## Job steps (order)

1. `dependency firewall` — `scripts/dep_firewall.py` (QUEUE #45).
2. `cargo fmt --all --check`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo test --workspace`
5. `schema validate --fixtures fixtures` (QUEUE #38)
6. `conformance run --profile C0`
7. `conformance run --profile C1`

## Job `conformance-c2` (QUEUE #117)

1. `conformance run --profile C2` — partial local protocol regression gate; separate job from C0/C1.

## Job `conformance-c3` (QUEUE #164)

1. `conformance run --profile C3` — local federation + CAP + export_deny regression.
2. **Not** a merge gate: do **not** add to branch-protection required checks.
3. Required checks stay: `fmt-clippy-test-schema-c0-c1`, `conformance-c2`.

## Profile C3 (QUEUE #141 scaffold; #153 governance)

Local federation ceremony (`run_c3`): **8** named cases — federation×4 + CAP + export_deny (#163) + `c3.crp.reject_node_route` (#167) + `c3.crp.route_candidate` (#170). Optional CI job `conformance-c3` (`#164`, non-gate). RFC-P process doc `#177` DONE; promotion-candidate fixtures `#178` DONE; non-operational gate `#179` DONE (RFC-0082); `run_c5` `#180` DONE (RFC-0083, not a CI job); promotion status `#181` DONE (Research remains RESEARCH); Reference v0.3 docs `#182` DONE; RFC-0077 `#183` DONE. Phase H closed; no OPEN. Phase I **DONE** (`#184`–`#198` @ RFC-0078). QUEUE I closed. Phase J **IN PROGRESS** (`#199` `#200` `#201` DONE; first OPEN `#202`). Settlement **PARTIAL** (`#176`); `run_c4` local (`#175`, not a CI job).

### Current posture (QUEUE #153 + `#164`)

| Surface | Status |
|---------|--------|
| CLI / library `run_c3` | available (8 named cases) |
| Job in `.github/workflows/ci.yml` | **`conformance-c3`** (QUEUE `#164`) — **informational** |
| Required branch-protection check | **no** — C3 is not a merge gate |
| Required checks remain | `fmt-clippy-test-schema-c0-c1`, `conformance-c2` only |

Manual / local run:

```bash
cargo run -p aira-cli -- --root "$ROOT" conformance run --profile C3 --out /tmp/aira-c3
```

### Optional CI job `conformance-c3` (`#164` **DONE**)

Criteria that **MUST** hold (and do) for the non-required workflow job named `conformance-c3`:

1. `run_c3` exposes a stable, documented set of **≥6** named local cases (**DONE** `#163`: 6; **`#167`/`#170`**: 8 with CRP).
2. Cases are **local-only** (no live WAN / discv5 / flake-prone peer mesh).
3. Job is **informational**: present in `ci.yml` but **not** listed in branch protection required checks.
4. Adding the job does **not** change C0/C1/C2 steps or required check names.
5. Docs (`ci-governance.md`, `conformance.md`) name the job and restate “not a merge gate”.

`#164` added the job; it remains **informational** (not in required status checks).

### When C3 may become a merge gate (future; not `#164`)

Optional job alone is insufficient. Promoting `conformance-c3` into **required** status checks requires a **separate** QUEUE atom + RFC that records:

1. Sustained green optional `conformance-c3` on `main` (operator judgment; no silent add).
2. C3 local profile covers the Conformance ladder slice intended for the gate (federation + capability advertisement; CRP local cases when those atoms are DONE).
3. Explicit update of the “Required status check” table and branch-protection checklist in this file.
4. C0/C1/C2 gates remain required and are not weakened.

Until that atom exists, treat any PR that adds C3 to required checks as **out of policy**.

### Anti-scope for C3 CI

```text
full Book II wire / production federation mesh
discv5 / ICE / TURN as merge-gate dependencies
CRP marketplace / settlement ledger / scheduler
```

## Job `cargo-deny` (QUEUE `#197`)

1. `cargo deny check` via SHA-pinned `EmbarkStudios/cargo-deny-action` and repo [`deny.toml`](../deny.toml).
2. **Not** a merge gate: do **not** add `cargo-deny` to branch-protection required checks (same posture as `conformance-c3`).
3. Informational supply-chain job (licenses / advisories / bans / sources). Required checks stay: `fmt-clippy-test-schema-c0-c1`, `conformance-c2`.
4. First-pass `deny.toml`: `unmaintained = "none"` (`rustls-pemfile` is still a direct TLS dep). Ignore `RUSTSEC-2026-0253` (`lru` via `rqrr`).

## External checks

GitGuardian may run as an additional PR check; it is **not** listed in `ci.yml` and is not part of the required check name above unless explicitly added to branch protection.

## Related

- Phase F plan (DONE): [`phase-f-plan.md`](phase-f-plan.md) F0
- Phase G plan (DONE): [`phase-g-plan.md`](phase-g-plan.md)
- Phase I plan (DONE `#198`): [`phase-i-plan.md`](phase-i-plan.md) — Handle/Verify/Policy/Reuse stabilization; RFC-0078; QUEUE I closed
- Phase H plan (DONE `#183`): [`phase-h-plan.md`](phase-h-plan.md) — RFC-0077; QUEUE H closed
- Phase C CI gate: [`phase-c-plan.md`](phase-c-plan.md) `#38`
- RFC: [`AIRA-RFC-0058`](../specs/rfc/AIRA-RFC-0058-ci-governance-doc.md); branch protection sync [`AIRA-RFC-0070`](../specs/rfc/AIRA-RFC-0070-ci-branch-protection-sync.md) (`#120`); MSRV + supply-chain [`AIRA-RFC-0095`](../specs/rfc/AIRA-RFC-0095-msrv-supply-chain-ci.md) (`#197`)
