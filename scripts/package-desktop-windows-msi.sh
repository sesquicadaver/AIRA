#!/usr/bin/env bash
# QUEUE #149 / Analyze-179 — Windows MSI pipeline helper (production path).
#
# Stages WiX sources + payload bins and either:
#   --layout-only  write a CI-friendly .tar.gz of the MSI stage (default on non-Windows)
#   --dry-run      print candle/light commands (no WiX required)
#   --execute      run WiX candle+light (Windows + WiX Toolset required)
#
# Out of scope: Authenticode / enterprise codesign.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

MODE="layout"
OUT_DIR="${PACKAGE_OUT:-$ROOT/release/desktop}"
VERSION="${AIRA_PACKAGE_VERSION:-$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')}"
# WiX Product/@Version is major.minor.build[.revision] with 0-65535 parts.
WIX_VERSION="${AIRA_MSI_VERSION:-${VERSION%.0}.0}"
ARCH="${PACKAGE_ARCH:-$(uname -m)}"
NAME="aira-desktop-windows-msi-${VERSION}-${ARCH}"
UPGRADE_CODE="${AIRA_MSI_UPGRADE_CODE:-A1B2C3D4-E5F6-4789-A012-3456789ABCDE}"
MANUFACTURER="${AIRA_MSI_MANUFACTURER:-AIRA}"
PRODUCT_NAME="${AIRA_MSI_PRODUCT_NAME:-AIRA Desktop}"
SKIP_BUILD=0
LAYOUT_BINS=0

usage() {
  cat <<'EOF'
Usage: scripts/package-desktop-windows-msi.sh [--layout-only|--dry-run|--execute] [options]

  --layout-only   Stage WiX + stub/real bins; write .tar.gz under release/desktop/ (CI)
  --dry-run       Stage sources and print candle/light commands (default with --layout-only notes)
  --execute       Run candle + light (requires Windows host + WiX on PATH)
  --skip-build    Use existing target/release/*.exe (with --execute / real bins)
  --stub-bins     Force stub .exe payload (CI / layout without cargo)
  --out DIR       Output directory (default: release/desktop)

Environment:
  PACKAGE_OUT / AIRA_PACKAGE_VERSION / PACKAGE_ARCH
  AIRA_MSI_VERSION       WiX Product Version (default derived from workspace)
  AIRA_MSI_UPGRADE_CODE  Stable UpgradeCode GUID
  AIRA_MSI_MANUFACTURER / AIRA_MSI_PRODUCT_NAME

Examples:
  ./scripts/package-desktop-windows-msi.sh --layout-only
  ./scripts/package-desktop-windows-msi.sh --dry-run --stub-bins
  ./scripts/package-desktop-windows-msi.sh --execute --skip-build
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --layout-only) MODE="layout"; LAYOUT_BINS=1; shift ;;
    --dry-run) MODE="dry-run"; shift ;;
    --execute) MODE="execute"; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --stub-bins) LAYOUT_BINS=1; shift ;;
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

STAGE="$OUT_DIR/$NAME"
PAYLOAD="$STAGE/payload"
BIN_DIR="$PAYLOAD/bin"
WXS_OUT="$STAGE/aira-desktop.wxs"
TARBALL="$OUT_DIR/${NAME}.tar.gz"
MSI_OUT="$OUT_DIR/${NAME}.msi"

echo "== package Desktop Windows MSI (${NAME}) mode=${MODE} =="
rm -rf "$STAGE"
mkdir -p "$BIN_DIR" "$STAGE/share/doc/aira"

write_stub_exe() {
  local dest="$1"
  cat >"$dest" <<'STUB'
@echo off
echo AIRA Desktop MSI stub payload (layout-only; rebuild on Windows) >&2
exit /b 1
STUB
}

stage_binaries() {
  if [[ "$LAYOUT_BINS" -eq 1 ]] || { [[ "$MODE" != "execute" ]] && ! is_windows_host; }; then
    LAYOUT_BINS=1
    for bin in aira aira-node aira-desktop; do
      write_stub_exe "$BIN_DIR/${bin}.exe"
    done
    return
  fi
  if [[ "$SKIP_BUILD" -eq 0 ]]; then
    echo "== cargo build --release (desktop) =="
    cargo build -p aira-cli -p aira-node -p aira-desktop --release
  fi
  for bin in aira aira-node aira-desktop; do
    src="$ROOT/target/release/${bin}.exe"
    if [[ ! -f "$src" ]]; then
      echo "missing binary: $src (build first, --skip-build, or --stub-bins)" >&2
      exit 1
    fi
    cp -f "$src" "$BIN_DIR/${bin}.exe"
  done
}

stage_binaries

# Render WiX (escape & in sed carefully — version is numeric-ish).
sed \
  -e "s/@VERSION@/${WIX_VERSION}/g" \
  -e "s/@UPGRADE_CODE@/${UPGRADE_CODE}/g" \
  -e "s/@MANUFACTURER@/${MANUFACTURER}/g" \
  -e "s/@PRODUCT_NAME@/${PRODUCT_NAME}/g" \
  "$ROOT/deploy/windows/aira-desktop.wxs" > "$WXS_OUT"

cp -f "$ROOT/docs/desktop-packaging-windows.md" "$STAGE/share/doc/aira/" 2>/dev/null || true

cat > "$STAGE/build-msi.bat" <<EOF
@echo off
REM Build MSI with WiX Toolset (candle + light) from this stage directory.
setlocal
cd /d "%~dp0"
where candle >nul 2>&1 || (echo candle.exe not on PATH — install WiX Toolset >&2 & exit /b 1)
where light >nul 2>&1 || (echo light.exe not on PATH — install WiX Toolset >&2 & exit /b 1)
candle -nologo -out aira-desktop.wixobj aira-desktop.wxs || exit /b 1
light -nologo -out "..\\${NAME}.msi" aira-desktop.wixobj || exit /b 1
echo OK: ..\\${NAME}.msi
EOF

cat > "$STAGE/README.md" <<EOF
# AIRA Desktop — Windows MSI stage (QUEUE #149)

**Not** a production distributed runtime. Authenticode is **Out**.

## CI / layout artifact

This directory (and the sibling \`.tar.gz\`) is produced by:

\`\`\`bash
./scripts/package-desktop-windows-msi.sh --layout-only
\`\`\`

Stub \`payload/bin/*.exe\` are for structure smoke. Rebuild bins on Windows for a real installer.

## Build MSI on Windows

1. Install [WiX Toolset](https://wixtoolset.org/) so \`candle\` / \`light\` are on PATH.
2. From this stage directory:

\`\`\`bat
build-msi.bat
\`\`\`

Or via the helper:

\`\`\`bash
./scripts/package-desktop-windows-msi.sh --execute --skip-build
\`\`\`

Docs: \`share/doc/aira/desktop-packaging-windows.md\`
EOF

cat > "$STAGE/MANIFEST.txt" <<EOF
name: ${NAME}
product: ${PRODUCT_NAME}
version: ${VERSION}
wix_version: ${WIX_VERSION}
arch: ${ARCH}
mode: ${MODE}
stub_bins: ${LAYOUT_BINS}
upgrade_code: ${UPGRADE_CODE}
built: $(date -u +%Y-%m-%dT%H:%M:%SZ)
wxs: aira-desktop.wxs
payload: payload/bin/{aira,aira-node,aira-desktop}.exe
EOF

mkdir -p "$OUT_DIR"

print_wix_commands() {
  echo "+ (cd $STAGE && candle -nologo -out aira-desktop.wixobj aira-desktop.wxs)"
  echo "+ (cd $STAGE && light -nologo -out $MSI_OUT aira-desktop.wixobj)"
}

case "$MODE" in
  layout)
    rm -f "$TARBALL"
    tar -C "$OUT_DIR" -czf "$TARBALL" "$NAME"
    echo "OK: $TARBALL"
    echo "stage: $STAGE"
    print_wix_commands
    ;;
  dry-run)
    print_wix_commands
    echo "dry-run complete — see docs/desktop-packaging-windows.md §MSI pipeline"
    echo "stage: $STAGE (not packed)"
    ;;
  execute)
    if ! is_windows_host; then
      echo "error: --execute requires a Windows host" >&2
      exit 1
    fi
    if ! command -v candle >/dev/null 2>&1 || ! command -v light >/dev/null 2>&1; then
      echo "error: candle/light not on PATH (install WiX Toolset)" >&2
      exit 1
    fi
    (
      cd "$STAGE"
      candle -nologo -out aira-desktop.wixobj aira-desktop.wxs
      light -nologo -out "$MSI_OUT" aira-desktop.wixobj
    )
    echo "OK: $MSI_OUT"
    ;;
  *)
    echo "internal error: mode=$MODE" >&2
    exit 2
    ;;
esac
