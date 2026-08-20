# Living Spec — Analyze-114 / #79

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Install без cargo | `scripts/package-desktop-linux.sh` + tarball `install.sh` | package smoke; bash -n |
| Autostart paths valid | `~/.local/bin` + `Exec=aira-desktop` | `tests/launcher.rs` autostart contract |
| Both `.desktop` | `deploy/desktop/*.desktop` + launcher-install | validate + install_menu_entries |
| Dev CLI notes | `docs/desktop-packaging.md` | doc review |
| RFC | RFC-0028 | — |
| Anti-stub | real bins in tarball; no pass | CODE_REVIEW |
