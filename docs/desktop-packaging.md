# AIRA Desktop — Linux packaging (QUEUE #79; deb `#150`)

**Status:** Developer Preview tarball + optional `.deb` production path. **Not** a production distributed AIRA runtime.  
**Phase G `#150`:** [`scripts/package-desktop-linux-deb.sh`](../scripts/package-desktop-linux-deb.sh) + [`deploy/linux/debian/control.in`](../deploy/linux/debian/control.in).  
**Format:** **tarball + `.desktop`** for Preview; **`.deb`** installs to `/usr/bin` + `/usr/share/applications` (still `Exec=aira` / `Exec=aira-desktop`). AppImage is **Out**.

## End-user install (no `cargo`)

```bash
# from a release artifact produced by scripts/package-desktop-linux.sh
tar -xzf aira-desktop-linux-*-*.tar.gz
cd aira-desktop-linux-*-*
./install.sh
```

Default prefix: `~/.local`

| Path | Contents |
|------|----------|
| `~/.local/bin/` | `aira`, `aira-node`, `aira-desktop` |
| `~/.local/share/applications/` | `aira.desktop`, `aira-desktop.desktop` |

Ensure `~/.local/bin` is on `PATH` (most desktop distros already do for login shells). Without it, menu/autostart cannot resolve bare `Exec=` names.

Then: application menu → **AIRA Desktop**, or run `aira-desktop`.

Uninstall:

```bash
./uninstall.sh
# or from the repo after packaging helpers are present:
bash deploy/desktop/uninstall-user.sh
```

Node data (`~/.local/share/aira`) and settings (`~/.config/aira`) are kept.

## Build the tarball (maintainers)

```bash
./scripts/package-desktop-linux.sh
# → release/desktop/aira-desktop-linux-<ver>-<arch>.tar.gz
```

`--skip-build` reuses existing `target/release/{aira,aira-node,aira-desktop}`.

## deb pipeline (`#150`)

Developer Preview remains the **tarball**. For a Debian package production path:

1. Stage (CI smoke — stub bins + control + `.stage.tar.gz`):

```bash
./scripts/package-desktop-linux-deb.sh --layout-only
# → release/desktop/aira-desktop_<ver>_<arch>.stage.tar.gz
```

2. Dry-run `dpkg-deb` (no package built):

```bash
./scripts/package-desktop-linux-deb.sh --dry-run --stub-bins
```

3. Build a real `.deb` (requires `dpkg-deb`):

```bash
./scripts/package-desktop-linux-deb.sh --execute --skip-build
# → release/desktop/aira-desktop_<ver>_<arch>.deb
sudo dpkg -i release/desktop/aira-desktop_*.deb
```

Install layout: `/usr/bin/{aira,aira-node,aira-desktop}` and `/usr/share/applications/{aira,aira-desktop}.desktop`. **Out:** AppImage.

## Developer (AIRA Dev) notes

| Goal | How |
|------|-----|
| Full CLI after package install | `aira` on PATH (`aira desktop …`, `aira identity`, peer, models, …) |
| Colocated Dev root | `aira desktop start --data-root /path` / `aira-desktop --data-root …` |
| Override node binary | `--node-bin` / `AIRA_NODE_BIN` |
| From source (no package) | `cargo build -p aira-cli -p aira-node -p aira-desktop --release` then `deploy/desktop/install-user.sh` |
| Menu only (bins already on PATH) | `aira desktop launcher-install` (both `.desktop` files) |

Packaging does **not** change Core / C0–C1. Peer is not started (P0 only).

## Autostart path contract (#78)

`autostart_on_login` writes `~/.config/autostart/aira-desktop.desktop` with `Exec=aira-desktop --from-autostart`.  
The **menu** launcher stays `Exec=aira-desktop` (always opens the window). Install must place `aira-desktop` on PATH (this package → `~/.local/bin`). Absolute AppImage paths were avoided so hooks do not diverge from the shared runtime.

## Related

- macOS packaging: [`desktop-packaging-macos.md`](desktop-packaging-macos.md)
- Windows packaging: [`desktop-packaging-windows.md`](desktop-packaging-windows.md)
- GUI: [`desktop-gui.md`](desktop-gui.md)
- Launcher: [`desktop-launcher.md`](desktop-launcher.md)
- UX / plan: [`desktop-ux.md`](desktop-ux.md), [`phase-e-plan.md`](phase-e-plan.md)
- RFC: [`AIRA-RFC-0028`](../specs/rfc/AIRA-RFC-0028-linux-desktop-packaging.md)
