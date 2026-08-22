#!/usr/bin/env bash
# Install AIRA Desktop macOS Developer Preview (.app bundle) (QUEUE #88).
#
# Layouts supported:
# 1) Extracted tarball (this script at package root next to AIRA Desktop.app/)
# 2) Repo: deploy/macos/install-user.sh with target/release binaries built
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="AIRA Desktop.app"
DEST="${INSTALL_DEST:-$HOME/Applications}"
REPO_RELEASE="$(cd "$HERE/../.." 2>/dev/null && pwd)/target/release"
BIN_DIR="${PREFIX:-$HOME/.local}/bin"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "install-user.sh is for macOS only (got $(uname -s))" >&2
  exit 1
fi

resolve_app_bundle() {
  if [[ -d "$HERE/$APP_NAME" ]]; then
    echo "$HERE/$APP_NAME"
    return
  fi
  if [[ -d "$HERE/../$APP_NAME" ]]; then
    echo "$HERE/../$APP_NAME"
    return
  fi
  echo "missing bundle: $APP_NAME (extract tarball or run package script)" >&2
  exit 1
}

ensure_bundle_bins() {
  local app="$1"
  local macos="$app/Contents/MacOS"
  mkdir -p "$macos"
  for name in aira aira-node aira-desktop; do
    if [[ -x "$macos/$name" ]]; then
      continue
    fi
    if [[ -x "$REPO_RELEASE/$name" ]]; then
      cp -f "$REPO_RELEASE/$name" "$macos/$name"
      chmod +x "$macos/$name"
      echo "staged $macos/$name from release build"
    else
      echo "missing binary for bundle: $name" >&2
      exit 1
    fi
  done
}

APP_SRC="$(resolve_app_bundle)"
mkdir -p "$DEST"
rm -rf "$DEST/$APP_NAME"
cp -R "$APP_SRC" "$DEST/$APP_NAME"
ensure_bundle_bins "$DEST/$APP_NAME"

mkdir -p "$BIN_DIR"
for name in aira aira-node aira-desktop; do
  ln -sf "$DEST/$APP_NAME/Contents/MacOS/$name" "$BIN_DIR/$name"
  echo "linked $BIN_DIR/$name"
done

echo "OK: installed $DEST/$APP_NAME"
echo "Open: Finder → Applications → AIRA Desktop"
echo "CLI:  aira desktop status  (via $BIN_DIR on PATH)"
