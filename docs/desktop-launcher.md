# Linux Desktop launcher (QUEUE #77)

**Status:** Developer Preview. GUI: [`desktop-gui.md`](desktop-gui.md). Packaging: Linux [`desktop-packaging.md`](desktop-packaging.md); macOS [`desktop-packaging-macos.md`](desktop-packaging-macos.md); Windows [`desktop-packaging-windows.md`](desktop-packaging-windows.md).

## Install

Requires `aira` and `aira-node` on `PATH` (or sibling binaries so `aira desktop start` can resolve `aira-node`). Prefer the Linux tarball install from [`desktop-packaging.md`](desktop-packaging.md) when you do not want `cargo`.

```bash
cargo build -p aira-cli -p aira-node -p aira-desktop --release
# put target/release on PATH, then:
aira desktop launcher-install
```

Installs both menu entries to:

```text
$XDG_DATA_HOME/applications/aira.desktop
$XDG_DATA_HOME/applications/aira-desktop.desktop
# default: ~/.local/share/applications/
```

Then open the application menu → **AIRA** (starts `aira desktop start`) or **AIRA Desktop** (GUI).

Manual copy (equivalent):

```bash
mkdir -p ~/.local/share/applications
cp deploy/desktop/aira.desktop deploy/desktop/aira-desktop.desktop ~/.local/share/applications/
# optional: update-desktop-database ~/.local/share/applications
```

## Stop

| Method | Command / UI |
|--------|----------------|
| Menu action | AIRA → **Stop AIRA** (`aira desktop stop`) |
| CLI | `aira desktop stop` |
| GUI | AIRA Desktop → Stop / Quit |

Autostart-on-login hooks: [`desktop-gui.md`](desktop-gui.md).

## Uninstall launcher

```bash
aira desktop launcher-uninstall
```

## Notes

- Entry uses `Terminal=false`; node logs go under the Desktop log dir (see phase-e §2.1).
- Peer is not started (P0 only).
- Global `aira --root` is ignored by `desktop` commands; use `--data-root` for Dev/tests.
