# AIRA-RFC-0058 — CI governance documentation

## 1. Summary

Phase F `#109`: `docs/ci-governance.md` documents required status check `fmt-clippy-test-schema-c0-c1`, workflow triggers, toolchain pin, and recommended `main` branch protection — aligned with `.github/workflows/ci.yml`.

## 5. Non-Goals

Changing GitHub branch protection via API; adding C2 to required checks (`#117`).

## 15. Tests

`cargo test -p aira-desktop-runtime --test ci_governance_doc`
