#!/usr/bin/env bash
# QUEUE #150 / Analyze-180 — Linux .deb production-path helper.
#
# Stages a Debian package tree (bins + .desktop) and either:
#   --layout-only  write CI-friendly .tar.gz of the stage (stub bins OK)
#   --dry-run      print dpkg-deb command (no root / no dpkg required)
#   --execute      run dpkg-deb -b (requires dpkg-deb on PATH)
#
# Out of scope: AppImage.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MODE="layout"
OUT_DIR="${PACKAGE_OUT:-$ROOT/release/desktop}"
VERSION="${AIRA_PACKAGE_VERSION:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')}"
ARCH_RAW="${PACKAGE_ARCH:-$(uname -m)}"
SKIP_BUILD=0
STUB_BINS=0

# Map uname -m → Debian arch.
case "$ARCH_RAW" in
  x86_64|amd64) DEB_ARCH="amd64" ;;
  aarch64|arm64) DEB_ARCH="arm64" ;;
  armv7*|armhf) DEB_ARCH="armhf" ;;
  *) DEB_ARCH="$ARCH_RAW" ;;
esac

NAME="aira-desktop_${VERSION}_${DEB_ARCH}"
STAGE="$OUT_DIR/${NAME}_stage"
DEB_ROOT="$STAGE/debroot"
DEB_OUT="$OUT_DIR/${NAME}.deb"
TARBALL="$OUT_DIR/${NAME}.stage.tar.gz"

usage() {
  cat <<'EOF'
Usage: scripts/package-desktop-linux-deb.sh [--layout-only|--dry-run|--execute] [options]

  --layout-only   Stage debroot + write .stage.tar.gz (CI package smoke)
  --dry-run       Stage and print dpkg-deb command
  --execute       Build .deb with dpkg-deb -b
  --skip-build    Use existing target/release binaries
  --stub-bins     Stub /usr/bin payloads (layout without cargo)
  --out DIR       Output directory (default: release/desktop)

Environment:
  PACKAGE_OUT / AIRA_PACKAGE_VERSION / PACKAGE_ARCH

Examples:
  ./scripts/package-desktop-linux-deb.sh --layout-only
  ./scripts/package-desktop-linux-deb.sh --dry-run --stub-bins
  ./scripts/package-desktop-linux-deb.sh --execute --skip-build
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --layout-only) MODE="layout"; STUB_BINS=1; shift ;;
    --dry-run) MODE="dry-run"; shift ;;
    --execute) MODE="execute"; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --stub-bins) STUB_BINS=1; shift ;;
    --out) OUT_DIR="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

# Recompute paths if --out changed after defaults.
STAGE="$OUT_DIR/${NAME}_stage"
DEB_ROOT="$STAGE/debroot"
DEB_OUT="$OUT_DIR/${NAME}.deb"
TARBALL="$OUT_DIR/${NAME}.stage.tar.gz"

echo "== package Desktop Linux deb (${NAME}) mode=${MODE} =="
rm -rf "$STAGE"
mkdir -p \
  "$DEB_ROOT/DEBIAN" \
  "$DEB_ROOT/usr/bin" \
  "$DEB_ROOT/usr/share/applications" \
  "$DEB_ROOT/usr/share/doc/aira-desktop" \
  "$STAGE/share/doc/aira"

write_stub_bin() {
  local dest="$1"
  cat >"$dest" <<'STUB'
#!/usr/bin/env bash
echo "AIRA Desktop deb stub payload (layout-only; rebuild with cargo)" >&2
exit 1
STUB
  chmod +x "$dest"
}

stage_binaries() {
  if [[ "$STUB_BINS" -eq 1 ]]; then
    for bin in aira aira-node aira-desktop; do
      write_stub_bin "$DEB_ROOT/usr/bin/$bin"
    done
    return
  fi
  if [[ "$SKIP_BUILD" -eq 0 ]]; then
    echo "== cargo build --release (desktop) =="
    cargo build -p aira-cli -p aira-node -p aira-desktop --release
  fi
  for bin in aira aira-node aira-desktop; do
    src="$ROOT/target/release/$bin"
    if [[ ! -x "$src" ]]; then
      echo "missing binary: $src (build first, --skip-build, or --stub-bins)" >&2
      exit 1
    fi
    cp -f "$src" "$DEB_ROOT/usr/bin/$bin"
    chmod 755 "$DEB_ROOT/usr/bin/$bin"
  done
}

stage_binaries

sed \
  -e "s/@VERSION@/${VERSION}/g" \
  -e "s/@ARCH@/${DEB_ARCH}/g" \
  "$ROOT/deploy/linux/debian/control.in" > "$DEB_ROOT/DEBIAN/control"

# Ensure trailing newline in control (dpkg requirement).
printf '\n' >> "$DEB_ROOT/DEBIAN/control"

cp -f "$ROOT/deploy/desktop/aira.desktop" "$DEB_ROOT/usr/share/applications/"
cp -f "$ROOT/deploy/desktop/aira-desktop.desktop" "$DEB_ROOT/usr/share/applications/"

cat > "$DEB_ROOT/usr/share/doc/aira-desktop/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: AIRA
Source: https://github.com/sesquicadaver/AIRA

Files: *
Copyright: AIRA contributors
License: Apache-2.0 or MIT
EOF

cp -f "$ROOT/docs/desktop-packaging.md" "$STAGE/share/doc/aira/" 2>/dev/null || true
cp -f "$ROOT/docs/desktop-packaging.md" "$DEB_ROOT/usr/share/doc/aira-desktop/" 2>/dev/null || true

cat > "$STAGE/README.md" <<EOF
# AIRA Desktop — Linux .deb stage (QUEUE #150)

**Not** a production distributed runtime. AppImage is **Out**.

## CI / layout artifact

\`\`\`bash
./scripts/package-desktop-linux-deb.sh --layout-only
# → release/desktop/aira-desktop_<ver>_<arch>.stage.tar.gz
\`\`\`

## Build .deb

\`\`\`bash
./scripts/package-desktop-linux-deb.sh --execute --skip-build
# or:
dpkg-deb --build debroot ../aira-desktop_${VERSION}_${DEB_ARCH}.deb
\`\`\`

Install: \`sudo dpkg -i aira-desktop_*.deb\` → \`/usr/bin/{aira,aira-node,aira-desktop}\` + menu entries.
EOF

cat > "$STAGE/MANIFEST.txt" <<EOF
name: ${NAME}
package: aira-desktop
version: ${VERSION}
arch: ${DEB_ARCH}
mode: ${MODE}
stub_bins: ${STUB_BINS}
built: $(date -u +%Y-%m-%dT%H:%M:%SZ)
debroot: debroot/
control: debroot/DEBIAN/control
bins: debroot/usr/bin/{aira,aira-node,aira-desktop}
desktop: debroot/usr/share/applications/
EOF

mkdir -p "$OUT_DIR"

print_dpkg_cmd() {
  echo "+ dpkg-deb --build $DEB_ROOT $DEB_OUT"
}

case "$MODE" in
  layout)
    rm -f "$TARBALL"
    tar -C "$OUT_DIR" -czf "$TARBALL" "$(basename "$STAGE")"
    echo "OK: $TARBALL"
    echo "stage: $STAGE"
    print_dpkg_cmd
    ;;
  dry-run)
    print_dpkg_cmd
    echo "dry-run complete — see docs/desktop-packaging.md §deb pipeline"
    echo "stage: $STAGE (not packed)"
    ;;
  execute)
    if ! command -v dpkg-deb >/dev/null 2>&1; then
      echo "error: dpkg-deb not on PATH" >&2
      exit 1
    fi
    # Debian requires root-owned paths in some contexts; --root-owner-group helps non-root.
    if dpkg-deb --help 2>&1 | grep -q root-owner-group; then
      dpkg-deb --root-owner-group --build "$DEB_ROOT" "$DEB_OUT"
    else
      dpkg-deb --build "$DEB_ROOT" "$DEB_OUT"
    fi
    echo "OK: $DEB_OUT"
    ;;
  *)
    echo "internal error: mode=$MODE" >&2
    exit 2
    ;;
esac
