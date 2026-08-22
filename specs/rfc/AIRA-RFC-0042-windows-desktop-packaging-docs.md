# AIRA-RFC-0042 — Windows Desktop packaging documentation

## 1. Summary

Canonical end-user and maintainer documentation for Windows Desktop Developer Preview packaging: `.zip` install without `cargo`, paths (#90), Startup autostart (#91), and zip layout (#92). Cross-links Linux/macOS sibling docs and E3 RFCs.

## 5. Non-Goals

MSI / codesign; App Store; changing packaging scripts beyond doc references.

## 7. Change

- `docs/desktop-packaging-windows.md`
- Cross-links: `docs/desktop-gui.md`, `docs/phase-e-plan.md`, `docs/desktop-packaging.md`
- `scripts/package-desktop-windows.sh` copies doc into zip `share/doc/aira/`
- Tests: doc contract in `windows_packaging.rs`

## 15. Tests

`cargo test -p aira-desktop-runtime --test windows_packaging`
