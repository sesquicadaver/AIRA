# AIRA-RFC-0037 — macOS Desktop `.app` packaging (tarball)

## 1. Summary

Ship AIRA Desktop Developer Preview for macOS as **`AIRA Desktop.app`** inside a versioned **`.tar.gz`** with `install.sh` / `uninstall.sh`. Binaries live in `Contents/MacOS/`; `Info.plist` sets `CFBundleExecutable=aira-desktop`. Prefer tarball over DMG so Linux CI can validate layout (`--layout-only`).

## 5. Non-Goals

DMG / `hdiutil`; codesign / notarize; App Store; Windows (`E3`). Docs: RFC-0038 / `desktop-packaging-macos.md` (`#89`).

## 7. Change

- `scripts/package-desktop-macos.sh`
- `deploy/macos/Info.plist`, `install-user.sh`, `uninstall-user.sh`
- Docs: [`desktop-packaging-macos.md`](../../docs/desktop-packaging-macos.md) (RFC-0038)
- Tests: `crates/aira-desktop-runtime/tests/macos_packaging.rs`

## 15. Tests

`cargo test -p aira-desktop-runtime --test macos_packaging`
