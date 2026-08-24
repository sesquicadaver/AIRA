# AIRA-RFC-0070 — CI branch protection checklist sync

## 1. Summary

Phase G `#120`: `docs/ci-governance.md` branch-protection checklist and contract tests assert both required GitHub status check names (`fmt-clippy-test-schema-c0-c1`, `conformance-c2`) match `.github/workflows/ci.yml` job `name:` fields.

## 5. Non-Goals

GitHub API automation for branch protection; changing workflow jobs or C2 semantics.

## 15. Tests

`cargo test -p aira-desktop-runtime ci_governance_doc`
