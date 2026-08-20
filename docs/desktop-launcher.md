# Linux Desktop launcher (QUEUE #77)

**Status:** Developer Preview. Tray/GUI is `#78`; packaging is `#79`.

## Install

Requires `aira` and `aira-node` on `PATH` (or sibling binaries so `aira desktop start` can resolve `aira-node`).

```bash
cargo build -p aira-cli -p aira-node --release
# put target/release on PATH, then:
aira desktop launcher-install
```

Installs [`deploy/desktop/aira.desktop`](../deploy/desktop/aira.desktop) to:

```text
$XDG_DATA_HOME/applications/aira.desktop
# default: ~/.local/share/applications/aira.desktop
```

Then open the application menu → **AIRA** (starts `aira desktop start`).

Manual copy (equivalent):

```bash
mkdir -p ~/.local/share/applications
cp deploy/desktop/aira.desktop ~/.local/share/applications/
# optional: update-desktop-database ~/.local/share/applications
```

## Stop

| Method | Command / UI |
|--------|----------------|
| Menu action | AIRA → **Stop AIRA** (`aira desktop stop`) |
| CLI | `aira desktop stop` |
| Status | `aira desktop status` |

Tray Quit arrives in `#78`. Autostart-on-login hooks are `#78` (not this launcher file).

## Uninstall launcher

```bash
aira desktop launcher-uninstall
```

## Notes

- Entry uses `Terminal=false`; node logs go under the Desktop log dir (see phase-e §2.1).
- Peer is not started (P0 only).
- Global `aira --root` is ignored by `desktop` commands; use `--data-root` for Dev/tests.
