# AIRA Desktop GUI (QUEUE #78 / #85)

**Status:** Phase E **DONE**. Phase G G4: peer lifecycle CI stable (`#131`–`#132`); invite QR camera (`#133`). UX: Work tab + uk/en chrome; `open_ui_on_start` cannot lock out Settings.

**Binary:** `aira-desktop`  
**CLI:** `aira desktop gui`  
**Tech:** Rust-only (`eframe`/`egui`); no Node.js/Tauri. File dialogs via `rfd`.

## Features

| Control | Behavior |
|---------|----------|
| Tabs | **Work** / **Node** / **Network** / **Settings** (default: Work) |
| Work | submit problem text → local `POST /v1/problems` (same path as `aira problem submit`); **Answer** = `result.result` plus `status` / `verification_status`; identifiers and full VRA JSON are collapsed Details |
| Language | Українська / English; sidecar `ui-prefs.json` next to settings (not settings schema) |
| Status | lifecycle label + pid/listen/instance |
| Peer status | P1/P2 supervised `peer listen` pid/addr; P2 dht+apply-book; P3 relay+TTL; P4 gossip in Advanced |
| Start / Stop / Refresh | shared `aira-desktop-runtime` |
| Network profile | P0 / P1 / P2 selector; P1–P4 `peer_listen` edit (default `127.0.0.1:9797`) |
| Advanced | P3 relay toggle + `relay_ttl_days`; P4 gossip toggle; mutex hint P3 vs P4 |
| Federation (P5) | Import signed descriptor JSON; membership status display |
| Discovery (P6 Dev) | STUN query, discv announce, discv FIND — explicit inputs only; no public STUN default |
| Settings | language; `open_ui_on_start` (**login autostart window only**); `autostart_on_login` |
| Friend invite | Export/Import JSON; Show/Export/Import QR PNG; **Scan QR (camera)** |
| Quit | stop node + close window |
| Autostart | Linux: XDG `~/.config/autostart/aira-desktop.desktop` (`Exec=aira-desktop --from-autostart`); macOS: LaunchAgent + `--from-autostart`; Windows: Startup `AIRA Desktop.bat` + `--from-autostart` |

**Model layer (not in this GUI yet):** Core does not host inference. Local models are Artifacts + Capabilities (`aira models scan|list|activate` — Phase D). Binding a local backend as an **Execution CSU** is Phase K [`phase-k-plan.md`](phase-k-plan.md) (`#209`–`#211` DONE; first OPEN `#212`). This Desktop slice still uses C1 `execution-basic` until `#213`+.

After profile/`peer_listen`/relay TTL change: **Stop → Start** to apply peer supervise.

Interactive launch (menu icon, `aira-desktop`, `aira desktop gui`) **always opens the window**. `open_ui_on_start=false` only skips the window when the process is started with `--from-autostart` (login hook). `--force-ui` always shows. Headless node without GUI: `aira desktop start`.

## Known gaps

- Peer / federation / invite **operation** strings are still English (chrome is uk/en).
- No system tray; closing the window with Quit also stops the node.
- Work tab uses loopback HTTP only (the supervised `aira-node`).

System tray StatusNotifier icon is **not** required for this slice: the native window is the primary UI.

## Run

```bash
cargo build -p aira-desktop -p aira-node -p aira-cli
./target/debug/aira-desktop --data-root /tmp/aira-gui --node-bin ./target/debug/aira-node
# or:
./target/debug/aira desktop gui --data-root /tmp/aira-gui --node-bin ./target/debug/aira-node
```

Menu entry (GUI): Linux — [`deploy/desktop/aira-desktop.desktop`](../deploy/desktop/aira-desktop.desktop) via `aira desktop launcher-install` or the Linux tarball ([`desktop-packaging.md`](desktop-packaging.md)). macOS — **AIRA Desktop.app** via tarball ([`desktop-packaging-macos.md`](desktop-packaging-macos.md)).

## Related

- Invite file/QR: [`desktop-invite.md`](desktop-invite.md)
- Peer supervise: [`desktop-peer.md`](desktop-peer.md)
- P6 discovery shortcuts: [`desktop-discovery.md`](desktop-discovery.md)
- Packaging: Linux [`desktop-packaging.md`](desktop-packaging.md); macOS [`desktop-packaging-macos.md`](desktop-packaging-macos.md); Windows [`desktop-packaging-windows.md`](desktop-packaging-windows.md)
- Launcher: [`desktop-launcher.md`](desktop-launcher.md)
- UX canon: [`desktop-ux.md`](desktop-ux.md)
- Plan: [`phase-e-plan.md`](phase-e-plan.md)
- Demo submit: [`demo.md`](demo.md)
