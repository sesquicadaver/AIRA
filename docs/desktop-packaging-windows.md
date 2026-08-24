# AIRA Desktop — Windows packaging (QUEUE #93)

**Status:** Developer Preview. **Not** a production distributed AIRA runtime.  
**Format:** versioned **`.zip`** with `bin/*.exe` (not MSI). Codesign / SmartScreen / Store are **Out** of E3.  
**Phase G `#144`:** MSI pipeline doc + build script (production path).

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

- **Content:** `start "" "<absolute path to aira-desktop.exe>"`
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

## Related

- Linux packaging: [`desktop-packaging.md`](desktop-packaging.md)
- macOS packaging: [`desktop-packaging-macos.md`](desktop-packaging-macos.md)
- GUI: [`desktop-gui.md`](desktop-gui.md)
- UX / plan: [`desktop-ux.md`](desktop-ux.md), [`phase-e-plan.md`](phase-e-plan.md) §4c
- RFC: [`AIRA-RFC-0039`](../specs/rfc/AIRA-RFC-0039-windows-desktop-paths.md) (paths), [`AIRA-RFC-0040`](../specs/rfc/AIRA-RFC-0040-windows-login-autostart.md) (Startup), [`AIRA-RFC-0041`](../specs/rfc/AIRA-RFC-0041-windows-desktop-zip-packaging.md) (zip), [`AIRA-RFC-0042`](../specs/rfc/AIRA-RFC-0042-windows-desktop-packaging-docs.md) (this doc)
