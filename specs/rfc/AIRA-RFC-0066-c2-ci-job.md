# AIRA-RFC-0066 — C2 CI conformance job

## 1. Summary

Phase F `#117`: GitHub Actions job `conformance-c2` runs `aira conformance run --profile C2` on push/PR to `main`/`develop`; documented in `docs/ci-governance.md`.

## 5. Non-Goals

C2 semantic expansion; changing C0/C1 gate; branch protection API automation.

## 15. Tests

`cargo run -p aira-cli -- conformance run --profile C2 --out /tmp/aira-c2`
CI workflow `.github/workflows/ci.yml` job `conformance-c2`
