#!/usr/bin/env bash
# Build a Linux Developer Preview tarball for AIRA Desktop (QUEUE #79 / Analyze-114).
#
# Choice: tarball + .desktop (not AppImage) so Exec=aira / Exec=aira-desktop stay
# PATH-based and remain compatible with XDG autostart from #78.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SKIP_BUILD=0
OUT_DIR="${PACKAGE_OUT:-$ROOT/release/desktop}"
VERSION="${AIRA_PACKAGE_VERSION:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')}"
ARCH="$(uname -m)"
NAME="aira-desktop-linux-${VERSION}-${ARCH}"

usage() {
  cat <<'EOF'
Usage: scripts/package-desktop-linux.sh [--skip-build] [--out DIR]

Builds release binaries (aira, aira-node, aira-desktop), packs a tarball under
release/desktop/ (gitignored), and writes install/uninstall helpers.

Environment:
  PACKAGE_OUT            Output directory (default: release/desktop)
  AIRA_PACKAGE_VERSION   Version stamp (default: workspace version)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    --out) OUT_DIR="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

STAGE="$OUT_DIR/$NAME"
TARBALL="$OUT_DIR/${NAME}.tar.gz"

echo "== package Desktop Linux (${NAME}) =="
rm -rf "$STAGE"
mkdir -p "$STAGE"/{bin,share/applications,share/doc/aira,scripts}

if [[ "$SKIP_BUILD" -eq 0 ]]; then
  echo "== cargo build --release (desktop) =="
  cargo build -p aira-cli -p aira-node -p aira-desktop --release
else
  echo "== skip-build: using existing target/release =="
fi

for bin in aira aira-node aira-desktop; do
  src="$ROOT/target/release/$bin"
  if [[ ! -x "$src" ]]; then
    echo "missing binary: $src (build first or drop --skip-build)" >&2
    exit 1
  fi
  cp -f "$src" "$STAGE/bin/"
done

cp -f "$ROOT/deploy/desktop/aira.desktop" "$STAGE/share/applications/"
cp -f "$ROOT/deploy/desktop/aira-desktop.desktop" "$STAGE/share/applications/"
cp -f "$ROOT/deploy/desktop/install-user.sh" "$STAGE/install.sh"
cp -f "$ROOT/deploy/desktop/uninstall-user.sh" "$STAGE/uninstall.sh"
chmod +x "$STAGE/install.sh" "$STAGE/uninstall.sh"

cp -f \
  "$ROOT/docs/desktop-packaging.md" \
  "$ROOT/docs/desktop-gui.md" \
  "$ROOT/docs/desktop-launcher.md" \
  "$ROOT/docs/desktop-ux.md" \
  "$ROOT/docs/phase-e-plan.md" \
  "$STAGE/share/doc/aira/" 2>/dev/null || true

cat > "$STAGE/README.md" <<EOF
# AIRA Desktop — Linux Developer Preview

**Not** a production distributed AIRA runtime. Local P0 loopback node + native GUI.

## Install (no \`cargo\`)

\`\`\`bash
./install.sh
# binaries → ~/.local/bin
# menu → ~/.local/share/applications
\`\`\`

Ensure \`~/.local/bin\` is on \`PATH\` so:

- menu \`Exec=aira desktop start\` / \`Exec=aira-desktop\` resolve
- XDG autostart (\`Exec=aira-desktop\` from Settings) keeps working after login

Then open **AIRA Desktop** from the application menu, or run \`aira-desktop\`.

Developer CLI: \`aira\` is on the same PATH (\`aira desktop start|stop|status|gui\`).

## Uninstall

\`\`\`bash
./uninstall.sh
\`\`\`

Docs: \`share/doc/aira/desktop-packaging.md\`
EOF

cat > "$STAGE/MANIFEST.txt" <<EOF
name: ${NAME}
product: AIRA Desktop (Developer Preview)
version: ${VERSION}
arch: ${ARCH}
built: $(date -u +%Y-%m-%dT%H:%M:%SZ)
binaries: aira, aira-node, aira-desktop
applications: aira.desktop, aira-desktop.desktop
layout: PREFIX=\${HOME}/.local (bin + share/applications)
autostart_compat: Exec=aira-desktop requires PREFIX/bin on PATH
EOF

mkdir -p "$OUT_DIR"
tar -C "$OUT_DIR" -czf "$TARBALL" "$NAME"
echo "OK: $TARBALL"
echo "stage: $STAGE"
