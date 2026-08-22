# Living Spec — Analyze-123 / #88

| ТЗ | Модуль | Тести |
|----|--------|-------|
| .app layout | `package-desktop-macos.sh` | layout-only smoke |
| Info.plist | `deploy/macos/Info.plist` | macos_info_plist_contract |
| Install | `deploy/macos/install-user.sh` | bash -n |
| Bundle id ↔ LaunchAgent | CFBundleIdentifier | launch_agent_label_matches |
| RFC | RFC-0037 | — |
