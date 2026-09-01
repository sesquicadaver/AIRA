# AIRA Desktop — Windows packaging (QUEUE #93; MSI `#149`)

**Status:** Developer Preview zip + optional MSI pipeline stage. **Not** a production distributed AIRA runtime.  
**Format:** versioned **`.zip`** with `bin/*.exe` for Preview; MSI via WiX stage (`#149`). Authenticode / enterprise codesign are **Out**.  
**Phase G `#149`:** [`scripts/package-desktop-windows-msi.sh`](../scripts/package-desktop-windows-msi.sh) + [`deploy/windows/aira-desktop.wxs`](../deploy/windows/aira-desktop.wxs).

## End-user install (no `cargo`)

```bat
REM from a release artifact produced by scripts/package-desktop-windows.sh
REM extract aira-desktop-windows-*-*.zip, then:
install.bat
```

Default locations:

| Path | Contents |
|------|----------|
| `%LOCALAPPDATA%\Programs\AIRA\` | `aira.exe`, `aira-node.exe`, `aira-desktop.exe` |

Add `%LOCALAPPDATA%\Programs\AIRA` to `PATH` for terminal CLI, or run binaries by full path.

Then: run `%LOCALAPPDATA%\Programs\AIRA\aira-desktop.exe` (Start menu shortcut is optional; not shipped in E3).

Uninstall:

```bat
uninstall.bat
REM or from the repo after packaging helpers are present:
deploy\windows\uninstall-user.bat
```

Node data (`%LOCALAPPDATA%\AIRA`) and settings (`%APPDATA%\AIRA`) are kept. Startup batch (`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\AIRA Desktop.bat`) is **not** removed by uninstall — disable autostart in Settings first, or delete the file manually.

## Build the zip (maintainers)

On **Windows**:

```bash
./scripts/package-desktop-windows.sh
# → release/desktop/aira-desktop-windows-<ver>-<arch>.zip
```

`--skip-build` reuses existing `target/release/{aira,aira-node,aira-desktop}.exe`.

On **Linux CI** (layout smoke only — stub binaries, not runnable on Windows):

```bash
./scripts/package-desktop-windows.sh --layout-only
```

## Developer (AIRA Dev) notes

| Goal | How |
|------|-----|
| Full CLI after package install | `aira.exe` on PATH (`aira desktop …`, `aira identity`, peer, models, …) |
| Colocated Dev root | `aira desktop start --data-root C:\path` / `aira-desktop.exe --data-root …` |
| Override node binary | `--node-bin` / `AIRA_NODE_BIN` |
| From source (no package) | `cargo build -p aira-cli -p aira-node -p aira-desktop --release` then `deploy\windows\install-user.bat` |
| Custom install dir | `set INSTALL_DEST=C:\Tools\AIRA` then `install.bat` |

Packaging does **not** change Core / C0–C1. Peer is optional (P1 via Settings).

## Windows paths contract (#90)

When running without `--data-root`, Desktop uses system layout:

| Role | Path |
|------|------|
| Node root | `%LOCALAPPDATA%\AIRA` |
| Settings | `%APPDATA%\AIRA\desktop-settings.json` |
| Runtime (PID/lock) | `%LOCALAPPDATA%\AIRA\runtime` |
| Logs | `%LOCALAPPDATA%\AIRA\logs` |

Implemented in `aira-desktop-runtime::paths::windows_for_profile`.

## Autostart / Startup contract (#91)

When `autostart_on_login=true` in Settings, the runtime writes:

`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\AIRA Desktop.bat`

- **Content:** `start "" "<absolute path to aira-desktop.exe>" --from-autostart`
- **Program resolution:** prefers sibling `aira-desktop.exe`, `where aira-desktop.exe`, or `%LOCALAPPDATA%\Programs\AIRA\aira-desktop.exe` after package install

Disable in Settings removes the batch file. No Registry Run keys in E3.

Linux XDG and macOS LaunchAgent hooks are unchanged and unrelated on Windows.

## Zip layout (#92)

```text
aira-desktop-windows-<ver>-<arch>/
  bin/
    aira.exe
    aira-node.exe
    aira-desktop.exe
  install.bat
  uninstall.bat
  README.md
  MANIFEST.txt
  share/doc/aira/   (selected docs)
```

## MSI pipeline (`#149`)

Developer Preview remains the **zip**. For a WiX MSI production path:

1. Stage sources (works on Linux CI — stub bins + rendered `.wxs` + `.tar.gz`):

```bash
./scripts/package-desktop-windows-msi.sh --layout-only
# → release/desktop/aira-desktop-windows-msi-<ver>-<arch>.tar.gz
```

2. Dry-run candle/light (no WiX install required):

```bash
./scripts/package-desktop-windows-msi.sh --dry-run --stub-bins
```

3. On **Windows** with [WiX Toolset](https://wixtoolset.org/) on PATH, build a real MSI:

```bash
./scripts/package-desktop-windows-msi.sh --execute --skip-build
# or from the stage: build-msi.bat
```

Payload installs per-user under `%LOCALAPPDATA%\Programs\AIRA\` (same as `install.bat`). **Out:** Authenticode signing / SmartScreen / Store.

## Related

- Linux packaging: [`desktop-packaging.md`](desktop-packaging.md)
- macOS packaging: [`desktop-packaging-macos.md`](desktop-packaging-macos.md)
- GUI: [`desktop-gui.md`](desktop-gui.md)
- UX / plan: [`desktop-ux.md`](desktop-ux.md), [`phase-e-plan.md`](phase-e-plan.md) §4c
- RFC: [`AIRA-RFC-0039`](../specs/rfc/AIRA-RFC-0039-windows-desktop-paths.md) (paths), [`AIRA-RFC-0040`](../specs/rfc/AIRA-RFC-0040-windows-login-autostart.md) (Startup), [`AIRA-RFC-0041`](../specs/rfc/AIRA-RFC-0041-windows-desktop-zip-packaging.md) (zip), [`AIRA-RFC-0042`](../specs/rfc/AIRA-RFC-0042-windows-desktop-packaging-docs.md) (this doc)
