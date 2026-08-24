# AIRA Desktop — macOS packaging (QUEUE #89)

**Status:** Developer Preview. **Not** a production distributed AIRA runtime.  
**Format:** **`AIRA Desktop.app`** inside a versioned **`.tar.gz`** (not DMG). Codesign / notarize / App Store are **Out** of E2.  
**Phase G `#143`:** codesign doc + notarization helper script (production path).

## End-user install (no `cargo`)

```bash
# from a release artifact produced by scripts/package-desktop-macos.sh
tar -xzf aira-desktop-macos-*-*.tar.gz
cd aira-desktop-macos-*-*
./install.sh
```

Default locations:

| Path | Contents |
|------|----------|
| `~/Applications/AIRA Desktop.app` | GUI bundle (`Contents/MacOS/{aira,aira-node,aira-desktop}`) |
| `~/.local/bin/` | Symlinks to `Contents/MacOS/*` (CLI on PATH) |

Ensure `~/.local/bin` is on `PATH` for terminal use and for LaunchAgent resolution when autostart is enabled from Settings.

Then: Finder → **Applications** → **AIRA Desktop**, or run `aira-desktop`.

Uninstall:

```bash
./uninstall.sh
# or from the repo after packaging helpers are present:
bash deploy/macos/uninstall-user.sh
```

Node data (`~/Library/Application Support/AIRA`) and settings (`~/Library/Preferences/AIRA`) are kept. LaunchAgent plist (`~/Library/LaunchAgents/ai.aira.desktop.plist`) is **not** removed by uninstall — disable autostart in Settings first, or delete the plist manually.

## Build the tarball (maintainers)

On **macOS** (Darwin):

```bash
./scripts/package-desktop-macos.sh
# → release/desktop/aira-desktop-macos-<ver>-<arch>.tar.gz
```

`--skip-build` reuses existing `target/release/{aira,aira-node,aira-desktop}`.

On **Linux CI** (layout smoke only — stub binaries, not runnable on macOS):

```bash
./scripts/package-desktop-macos.sh --layout-only
```

## Developer (AIRA Dev) notes

| Goal | How |
|------|-----|
| Full CLI after package install | `aira` on PATH (`aira desktop …`, `aira identity`, peer, models, …) |
| Colocated Dev root | `aira desktop start --data-root /path` / `aira-desktop --data-root …` |
| Override node binary | `--node-bin` / `AIRA_NODE_BIN` |
| From source (no package) | `cargo build -p aira-cli -p aira-node -p aira-desktop --release` then `deploy/macos/install-user.sh` |
| Install `.app` only | `INSTALL_DEST=~/Applications deploy/macos/install-user.sh` (repo or tarball layout) |

Packaging does **not** change Core / C0–C1. Peer is optional (P1 via Settings).

## macOS paths contract (#86)

When running without `--data-root`, Desktop uses system layout:

| Role | Path |
|------|------|
| Node root | `~/Library/Application Support/AIRA` |
| Settings | `~/Library/Preferences/AIRA/desktop-settings.json` |
| Runtime (PID/lock) | `~/Library/Application Support/AIRA/runtime` |
| Logs | `~/Library/Logs/AIRA` |

Implemented in `aira-desktop-runtime::paths::macos_for_home`.

## Autostart / LaunchAgent contract (#87)

When `autostart_on_login=true` in Settings, the runtime writes:

`~/Library/LaunchAgents/ai.aira.desktop.plist`

- **Label:** `ai.aira.desktop` (matches `CFBundleIdentifier` in `Info.plist`)
- **Program:** absolute path to `aira-desktop` (resolved from bundle `Contents/MacOS` when installed via this package)
- **RunAtLoad:** `true`; **KeepAlive:** `false`

Disable in Settings removes the plist. The runtime does **not** call `launchctl bootstrap` — login session loads LaunchAgents automatically.

Linux XDG autostart (`~/.config/autostart/aira-desktop.desktop`) is unchanged and unrelated on macOS.

## Bundle layout (#88)

```text
AIRA Desktop.app/
  Contents/
    Info.plist          CFBundleExecutable=aira-desktop, CFBundleIdentifier=ai.aira.desktop
    MacOS/
      aira
      aira-node
      aira-desktop
```

Tarball root also includes `install.sh`, `uninstall.sh`, `README.md`, `MANIFEST.txt`, and `share/doc/aira/` (selected docs).

## Related

- Linux packaging: [`desktop-packaging.md`](desktop-packaging.md)
- GUI: [`desktop-gui.md`](desktop-gui.md)
- UX / plan: [`desktop-ux.md`](desktop-ux.md), [`phase-e-plan.md`](phase-e-plan.md) §4b
- RFC: [`AIRA-RFC-0035`](../specs/rfc/AIRA-RFC-0035-macos-desktop-paths.md) (paths), [`AIRA-RFC-0036`](../specs/rfc/AIRA-RFC-0036-macos-launchagent-autostart.md) (LaunchAgent), [`AIRA-RFC-0037`](../specs/rfc/AIRA-RFC-0037-macos-desktop-app-packaging.md) (`.app` tarball), [`AIRA-RFC-0038`](../specs/rfc/AIRA-RFC-0038-macos-desktop-packaging-docs.md) (this doc)
