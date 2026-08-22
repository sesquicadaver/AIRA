#!/usr/bin/env bash
# Build a macOS Developer Preview .app + tarball for AIRA Desktop (QUEUE #88 / Analyze-123).
#
# Choice: .app directory + .tar.gz (not DMG) so CI/Linux can validate layout without hdiutil.
# LaunchAgent (#87) resolves absolute Contents/MacOS/aira-desktop when autostart is enabled from the GUI.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SKIP_BUILD=0
LAYOUT_ONLY=0
OUT_DIR="${PACKAGE_OUT:-$ROOT/release/desktop}"
VERSION="${AIRA_PACKAGE_VERSION:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')}"
ARCH="${PACKAGE_ARCH:-$(uname -m)}"
APP_NAME="AIRA Desktop.app"
NAME="aira-desktop-macos-${VERSION}-${ARCH}"

usage() {
  cat <<'EOF'
Usage: scripts/package-desktop-macos.sh [--skip-build] [--layout-only] [--out DIR]

Builds release binaries (aira, aira-node, aira-desktop) on macOS, stages
AIRA Desktop.app/Contents/{MacOS,Info.plist}, and writes a .tar.gz under release/desktop/.

  --layout-only   Stub MacOS binaries (Linux CI layout smoke; not runnable on macOS)
  --skip-build    Use existing target/release binaries (macOS dev)

Environment:
  PACKAGE_OUT            Output directory (default: release/desktop)
  AIRA_PACKAGE_VERSION   Version stamp (default: workspace version)
  PACKAGE_ARCH           Arch label in artifact name (default: uname -m)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    --layout-only) LAYOUT_ONLY=1; SKIP_BUILD=1; shift ;;
    --out) OUT_DIR="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ "$LAYOUT_ONLY" -eq 0 ]] && [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS packaging requires Darwin host (or use --layout-only for structure smoke)" >&2
  exit 1
fi

STAGE="$OUT_DIR/$NAME"
TARBALL="$OUT_DIR/${NAME}.tar.gz"
APP_STAGE="$STAGE/$APP_NAME"
MACOS_DIR="$APP_STAGE/Contents/MacOS"

echo "== package Desktop macOS (${NAME}) =="
rm -rf "$STAGE"
mkdir -p "$MACOS_DIR" "$STAGE/share/doc/aira"

write_info_plist() {
  sed "s/@VERSION@/${VERSION}/g" "$ROOT/deploy/macos/Info.plist" > "$APP_STAGE/Contents/Info.plist"
}

write_stub_bin() {
  local dest="$1"
  cat >"$dest" <<'STUB'
#!/usr/bin/env bash
echo "AIRA Desktop Developer Preview stub (layout-only artifact; rebuild on macOS)" >&2
exit 1
STUB
  chmod +x "$dest"
}

stage_binaries() {
  if [[ "$LAYOUT_ONLY" -eq 1 ]]; then
    for bin in aira aira-node aira-desktop; do
      write_stub_bin "$MACOS_DIR/$bin"
    done
    return
  fi
  if [[ "$SKIP_BUILD" -eq 0 ]]; then
    echo "== cargo build --release (desktop) =="
    cargo build -p aira-cli -p aira-node -p aira-desktop --release
  else
    echo "== skip-build: using existing target/release =="
  fi
  for bin in aira aira-node aira-desktop; do
    src="$ROOT/target/release/$bin"
    if [[ ! -x "$src" ]]; then
      echo "missing binary: $src (build first or use --layout-only)" >&2
      exit 1
    fi
    cp -f "$src" "$MACOS_DIR/$bin"
  done
}

write_info_plist
stage_binaries

cp -f "$ROOT/deploy/macos/install-user.sh" "$STAGE/install.sh"
cp -f "$ROOT/deploy/macos/uninstall-user.sh" "$STAGE/uninstall.sh"
chmod +x "$STAGE/install.sh" "$STAGE/uninstall.sh"

cp -f \
  "$ROOT/docs/desktop-packaging-macos.md" \
  "$ROOT/docs/desktop-gui.md" \
  "$ROOT/docs/desktop-ux.md" \
  "$ROOT/docs/phase-e-plan.md" \
  "$STAGE/share/doc/aira/" 2>/dev/null || true

cat > "$STAGE/README.md" <<EOF
# AIRA Desktop — macOS Developer Preview

**Not** a production distributed AIRA runtime. Local P0/P1 loopback node + native GUI.

## Install (no \`cargo\`)

\`\`\`bash
./install.sh
# → ~/Applications/AIRA Desktop.app
# → symlinks in ~/.local/bin (aira, aira-node, aira-desktop)
\`\`\`

Double-click **AIRA Desktop** in Applications, or run \`aira-desktop\`.

Developer CLI: \`aira desktop start|stop|status|gui\`.

## Uninstall

\`\`\`bash
./uninstall.sh
\`\`\`

Node data (\`~/Library/Application Support/AIRA\`) and settings are kept.

Docs: \`share/doc/aira/desktop-packaging-macos.md\`

Layout-only artifacts (stub binaries) are for CI smoke only — rebuild on macOS for real use.
EOF

cat > "$STAGE/MANIFEST.txt" <<EOF
name: ${NAME}
product: AIRA Desktop (Developer Preview)
version: ${VERSION}
arch: ${ARCH}
built: $(date -u +%Y-%m-%dT%H:%M:%SZ)
bundle: ${APP_NAME}
binaries: Contents/MacOS/{aira,aira-node,aira-desktop}
layout_only: ${LAYOUT_ONLY}
install: ./install.sh → ~/Applications/${APP_NAME}
cli_symlinks: ~/.local/bin
launchagent_compat: absolute Contents/MacOS/aira-desktop when enabled from GUI
EOF

mkdir -p "$OUT_DIR"
tar -C "$OUT_DIR" -czf "$TARBALL" "$NAME"
echo "OK: $TARBALL"
echo "stage: $STAGE"
