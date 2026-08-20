#!/usr/bin/env bash
# Install AIRA Desktop Linux Developer Preview into the user prefix (QUEUE #79).
#
# Layouts supported:
# 1) Extracted tarball (this script at package root next to bin/ + share/)
# 2) Repo: deploy/desktop/install-user.sh with target/release binaries built
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
PREFIX="${PREFIX:-${HOME}/.local}"
BIN_DIR="$PREFIX/bin"
APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
REPO_RELEASE="$(cd "$HERE/../.." 2>/dev/null && pwd)/target/release"

mkdir -p "$BIN_DIR" "$APPS_DIR"

resolve_bin() {
  local name="$1"
  if [[ -x "$HERE/bin/$name" ]]; then
    echo "$HERE/bin/$name"
  elif [[ -x "$REPO_RELEASE/$name" ]]; then
    echo "$REPO_RELEASE/$name"
  elif command -v "$name" >/dev/null 2>&1; then
    command -v "$name"
  else
    echo "missing binary: $name (package bin/ or cargo build --release)" >&2
    exit 1
  fi
}

resolve_desktop() {
  local name="$1"
  if [[ -f "$HERE/share/applications/$name" ]]; then
    echo "$HERE/share/applications/$name"
  elif [[ -f "$HERE/$name" ]]; then
    echo "$HERE/$name"
  else
    echo "missing desktop entry: $name" >&2
    exit 1
  fi
}

for name in aira aira-node aira-desktop; do
  src="$(resolve_bin "$name")"
  cp -f "$src" "$BIN_DIR/$name"
  chmod +x "$BIN_DIR/$name"
  echo "installed $BIN_DIR/$name"
done

for name in aira.desktop aira-desktop.desktop; do
  src="$(resolve_desktop "$name")"
  cp -f "$src" "$APPS_DIR/$name"
  echo "installed $APPS_DIR/$name"
done

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPS_DIR" >/dev/null 2>&1 || true
fi

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *)
    echo "WARNING: $BIN_DIR is not on PATH." >&2
    echo "Add it so menu/autostart Exec=aira-desktop resolves (required for #78 hooks)." >&2
    echo "  export PATH=\"$BIN_DIR:\$PATH\"" >&2
    ;;
esac

echo "OK: AIRA Desktop installed under $PREFIX"
echo "Open: application menu → AIRA Desktop   or: aira-desktop"
echo "CLI:  aira desktop status"
