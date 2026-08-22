#!/usr/bin/env bash
# Build a Windows Developer Preview zip for AIRA Desktop (QUEUE #92 / Analyze-127).
#
# Choice: versioned .zip + bin/*.exe (not MSI) so Linux CI can validate layout (--layout-only).
# Startup autostart (#91) resolves absolute %LOCALAPPDATA%\Programs\AIRA\aira-desktop.exe when enabled.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SKIP_BUILD=0
LAYOUT_ONLY=0
OUT_DIR="${PACKAGE_OUT:-$ROOT/release/desktop}"
VERSION="${AIRA_PACKAGE_VERSION:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')}"
ARCH="${PACKAGE_ARCH:-$(uname -m)}"
NAME="aira-desktop-windows-${VERSION}-${ARCH}"

usage() {
  cat <<'EOF'
Usage: scripts/package-desktop-windows.sh [--skip-build] [--layout-only] [--out DIR]

Builds release binaries (aira.exe, aira-node.exe, aira-desktop.exe) on Windows,
stages bin/ + install helpers, and writes a .zip under release/desktop/.

  --layout-only   Stub .exe files (Linux CI layout smoke; not runnable on Windows)
  --skip-build    Use existing target/release binaries (Windows dev)

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

is_windows_host() {
  case "$(uname -s 2>/dev/null || echo unknown)" in
    MINGW*|MSYS*|CYGWIN*|Windows*) return 0 ;;
    *) return 1 ;;
  esac
}

if [[ "$LAYOUT_ONLY" -eq 0 ]] && ! is_windows_host; then
  echo "Windows packaging requires Windows host (or use --layout-only for structure smoke)" >&2
  exit 1
fi

STAGE="$OUT_DIR/$NAME"
ZIP="$OUT_DIR/${NAME}.zip"
BIN_DIR="$STAGE/bin"

echo "== package Desktop Windows (${NAME}) =="
rm -rf "$STAGE"
mkdir -p "$BIN_DIR" "$STAGE/share/doc/aira"

write_stub_exe() {
  local dest="$1"
  cat >"$dest" <<'STUB'
@echo off
echo AIRA Desktop Developer Preview stub (layout-only artifact; rebuild on Windows) >&2
exit /b 1
STUB
}

stage_binaries() {
  if [[ "$LAYOUT_ONLY" -eq 1 ]]; then
    for bin in aira aira-node aira-desktop; do
      write_stub_exe "$BIN_DIR/${bin}.exe"
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
    src="$ROOT/target/release/${bin}.exe"
    if [[ ! -f "$src" ]]; then
      echo "missing binary: $src (build first or use --layout-only)" >&2
      exit 1
    fi
    cp -f "$src" "$BIN_DIR/${bin}.exe"
  done
}

stage_binaries

cp -f "$ROOT/deploy/windows/install-user.bat" "$STAGE/install.bat"
cp -f "$ROOT/deploy/windows/uninstall-user.bat" "$STAGE/uninstall.bat"

cp -f \
  "$ROOT/docs/desktop-packaging-windows.md" \
  "$ROOT/docs/desktop-gui.md" \
  "$ROOT/docs/desktop-ux.md" \
  "$ROOT/docs/phase-e-plan.md" \
  "$STAGE/share/doc/aira/" 2>/dev/null || true

cat > "$STAGE/README.md" <<EOF
# AIRA Desktop — Windows Developer Preview

**Not** a production distributed AIRA runtime. Local P0/P1 loopback node + native GUI.

## Install (no \`cargo\`)

\`\`\`bat
install.bat
REM → %LOCALAPPDATA%\\Programs\\AIRA\\{aira,aira-node,aira-desktop}.exe
\`\`\`

Run **aira-desktop.exe** from that folder (or add to PATH).

Developer CLI: \`aira.exe desktop start|stop|status|gui\`.

## Uninstall

\`\`\`bat
uninstall.bat
\`\`\`

Node data (\`%LOCALAPPDATA%\\AIRA\`) and settings are kept.

Docs: \`share/doc/aira/desktop-packaging-windows.md\`

Layout-only artifacts (stub binaries) are for CI smoke only — rebuild on Windows for real use.
EOF

cat > "$STAGE/MANIFEST.txt" <<EOF
name: ${NAME}
product: AIRA Desktop (Developer Preview)
version: ${VERSION}
arch: ${ARCH}
built: $(date -u +%Y-%m-%dT%H:%M:%SZ)
binaries: bin/{aira,aira-node,aira-desktop}.exe
layout_only: ${LAYOUT_ONLY}
install: install.bat → %LOCALAPPDATA%\\Programs\\AIRA
startup_compat: absolute Programs\\AIRA\\aira-desktop.exe when autostart enabled from GUI
EOF

mkdir -p "$OUT_DIR"
rm -f "$ZIP"
(
  cd "$OUT_DIR"
  zip -r -q "$ZIP" "$NAME"
)
echo "OK: $ZIP"
echo "stage: $STAGE"
