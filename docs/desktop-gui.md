# AIRA Desktop GUI (QUEUE #78 / #85)

**Binary:** `aira-desktop`  
**CLI:** `aira desktop gui`  
**Tech:** Rust-only (`eframe`/`egui`); no Node.js/Tauri. File dialogs via `rfd`.

## Features

| Control | Behavior |
|---------|----------|
| Status | lifecycle label + pid/listen/instance |
| Peer status | P1/P2 supervised `peer listen` pid/addr; P2 shows dht+apply-book; P3 relay+TTL in Advanced |
| Start / Stop / Refresh | shared `aira-desktop-runtime` |
| Network profile | P0 / P1 / P2 selector; P1/P2/P3 `peer_listen` edit (default `127.0.0.1:9797`) |
| Advanced | P3 relay hub toggle; `relay_ttl_days` (default 31); mutex hint P3 vs P4 gossip |
| Settings | `open_ui_on_start`, `autostart_on_login` (persisted) |
| Friend invite | Export/Import JSON; Show/Export/Import QR PNG (no camera) |
| Quit | stop node + close window |
| Autostart | Linux: XDG `~/.config/autostart/aira-desktop.desktop`; macOS: LaunchAgent `~/Library/LaunchAgents/ai.aira.desktop.plist` (`#87`); Windows: Startup `AIRA Desktop.bat` (`#91`) |

After profile/`peer_listen`/relay TTL change: **Stop → Start** to apply peer supervise.

System tray StatusNotifier icon is **not** required for this slice: the native window is the primary UI. Autostart launches `aira-desktop`.

## Run

```bash
cargo build -p aira-desktop -p aira-node -p aira-cli
./target/debug/aira-desktop --data-root /tmp/aira-gui --node-bin ./target/debug/aira-node
# or:
./target/debug/aira desktop gui --data-root /tmp/aira-gui --node-bin ./target/debug/aira-node
```

Menu entry (GUI): Linux — [`deploy/desktop/aira-desktop.desktop`](../deploy/desktop/aira-desktop.desktop) via `aira desktop launcher-install` or the Linux tarball ([`desktop-packaging.md`](desktop-packaging.md)). macOS — **AIRA Desktop.app** via tarball ([`desktop-packaging-macos.md`](desktop-packaging-macos.md)).

Headless start without UI: set `open_ui_on_start=false` in settings, or use `aira desktop start`.

## Related

- Invite file/QR: [`desktop-invite.md`](desktop-invite.md)
- Peer supervise: [`desktop-peer.md`](desktop-peer.md)
- Packaging: Linux [`desktop-packaging.md`](desktop-packaging.md); macOS [`desktop-packaging-macos.md`](desktop-packaging-macos.md); Windows [`desktop-packaging-windows.md`](desktop-packaging-windows.md)
- Launcher: [`desktop-launcher.md`](desktop-launcher.md)
- UX canon: [`desktop-ux.md`](desktop-ux.md)
- Plan: [`phase-e-plan.md`](phase-e-plan.md)
