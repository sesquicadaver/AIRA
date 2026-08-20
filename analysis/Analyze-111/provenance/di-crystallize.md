# Provenance — Analyze-111

1. New crate `aira-desktop-runtime` (not CLI-only module) for future tray reuse.
2. Auth: bearer_token only; desktop_ipc reserved.
3. Global `aira --root` ignored by `desktop` — use `--data-root` for Dev/tests.
4. No OS autostart in this atom (`#78`).
