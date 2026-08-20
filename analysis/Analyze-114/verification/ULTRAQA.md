# UltraQA — Analyze-114

Hostile checks:

1. Install without cargo → tarball `install.sh` places bins + menus under user prefix.
2. Autostart still `Exec=aira-desktop` → packaging keeps bare name; PREFIX/bin on PATH.
3. Uninstall removes bins/menus/autostart hook; keeps data dirs.
4. Out of scope: AppImage, macOS, Windows, P1 — not shipped.

Verdict: **PASS** (local package + install/uninstall smoke).
