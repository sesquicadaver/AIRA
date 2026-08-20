# AIRA-RFC-0028 — Linux Desktop packaging (tarball)

## 1. Summary

Ship AIRA Desktop Developer Preview for Linux as a **versioned tarball** with `aira`, `aira-node`, `aira-desktop`, Freedesktop `.desktop` files, and user-prefix `install.sh` / `uninstall.sh`. Prefer tarball over AppImage so `#78` autostart and menu `Exec=` names remain PATH-based.

## 5. Non-Goals

macOS DMG; Windows installer; AppImage tooling; P1 peer; changing Core.

## 7. Change

- `scripts/package-desktop-linux.sh`
- `deploy/desktop/install-user.sh`, `uninstall-user.sh`
- `aira desktop launcher-install` installs both menu entries
- Docs: `docs/desktop-packaging.md`

## 15. Tests

Launcher tests for both `.desktop` embeds; autostart Exec name contract; `bash -n` on packaging scripts; package smoke with `--skip-build` when release bins exist.
