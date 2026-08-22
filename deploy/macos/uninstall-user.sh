#!/usr/bin/env bash
# Remove AIRA Desktop .app and user bin symlinks (QUEUE #88).
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="AIRA Desktop.app"
DEST="${INSTALL_DEST:-$HOME/Applications}"
BIN_DIR="${PREFIX:-$HOME/.local}/bin"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "uninstall-user.sh is for macOS only (got $(uname -s))" >&2
  exit 1
fi

if [[ -d "$DEST/$APP_NAME" ]]; then
  rm -rf "$DEST/$APP_NAME"
  echo "removed $DEST/$APP_NAME"
else
  echo "not installed: $DEST/$APP_NAME"
fi

for name in aira aira-node aira-desktop; do
  link="$BIN_DIR/$name"
  if [[ -L "$link" ]] && [[ "$(readlink "$link")" == *"$APP_NAME/Contents/MacOS/$name" ]]; then
    rm -f "$link"
    echo "removed $link"
  fi
done

echo "OK: AIRA Desktop uninstalled (data under ~/Library kept)"
