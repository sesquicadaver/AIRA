# Contributing

## Scope rules (MVP freeze)

Until Issue #80 (MVP alpha acceptance), do **not** add:

- real federation / multi-node CRP
- GPU execution / LLM inference
- blockchain settlement
- PHM / Research Plane / web UI / cloud / Kubernetes

Allowed: bug fixes, schema/tests/docs, local deterministic C0/C1 work, partial local C2 stubs.

## Workflow

1. Branch from `develop` (or `main` for hotfixes).
2. Keep Core free of domain/ML/GPU/network routing logic.
3. Prefer Artifacts + Events + Policy over hidden state.
4. Include tests for behavioral changes.
5. Run locally:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Specs

Canonical sources live in `Manifesto etc/` (immutable). Implementation reads `specs/` copies. Do not edit `Manifesto etc/` or `Meditation_About/` in routine PRs.
