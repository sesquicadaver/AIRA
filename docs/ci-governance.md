# CI governance (QUEUE #109)

Canonical mapping between GitHub Actions and merge policy for `main`. Workflow source: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

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

## Workflow triggers

```text
push:    main, develop
pull_request: main, develop
```

## Toolchain

- Rust **1.94.0** (`rust-toolchain.toml` + `dtolnay/rust-toolchain@1.94.0` in CI).
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

## External checks

GitGuardian may run as an additional PR check; it is **not** listed in `ci.yml` and is not part of the required check name above unless explicitly added to branch protection.

## Related

- Phase F plan: [`phase-f-plan.md`](phase-f-plan.md) F0
- Phase C CI gate: [`phase-c-plan.md`](phase-c-plan.md) `#38`
- RFC: [`AIRA-RFC-0058`](../specs/rfc/AIRA-RFC-0058-ci-governance-doc.md)
