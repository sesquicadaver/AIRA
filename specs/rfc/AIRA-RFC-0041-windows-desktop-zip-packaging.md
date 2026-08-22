# AIRA-RFC-0041 — Windows Desktop zip packaging

## 1. Summary

Ship AIRA Desktop Developer Preview for Windows as a versioned **`.zip`** with `bin/{aira,aira-node,aira-desktop}.exe`, `install.bat` / `uninstall.bat`, and docs. Prefer zip over MSI so Linux CI can validate layout (`--layout-only`).

## 5. Non-Goals

MSI / NSIS; codesign / SmartScreen; App Store; macOS/Linux packaging. Docs: RFC-0042 / `desktop-packaging-windows.md` (`#93`).

## 7. Change

- `scripts/package-desktop-windows.sh`
- `deploy/windows/install-user.bat`, `uninstall-user.bat`
- Tests: `crates/aira-desktop-runtime/tests/windows_packaging.rs`

## 15. Tests

`cargo test -p aira-desktop-runtime --test windows_packaging`
