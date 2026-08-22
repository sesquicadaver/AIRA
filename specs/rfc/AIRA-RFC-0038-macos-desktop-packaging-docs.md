# AIRA-RFC-0038 — macOS Desktop packaging documentation

## 1. Summary

Canonical end-user and maintainer documentation for macOS Desktop Developer Preview packaging: `.app` + `.tar.gz` install without `cargo`, paths (#86), LaunchAgent autostart (#87), and bundle layout (#88). Cross-links Linux sibling doc and E2 RFCs.

## 5. Non-Goals

DMG / notarize; Windows (`E3`); App Store; changing packaging scripts beyond doc references.

## 7. Change

- `docs/desktop-packaging-macos.md`
- Cross-links: `docs/desktop-gui.md`, `docs/phase-e-plan.md`, `docs/desktop-packaging.md`
- `scripts/package-desktop-macos.sh` copies doc into tarball `share/doc/aira/`
- Tests: doc contract in `macos_packaging.rs`

## 15. Tests

`cargo test -p aira-desktop-runtime --test macos_packaging`
