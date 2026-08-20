#!/usr/bin/env bash
# Remove AIRA Desktop user-prefix install (QUEUE #79). Does not delete node data/settings.
set -euo pipefail

PREFIX="${PREFIX:-${HOME}/.local}"
BIN_DIR="$PREFIX/bin"
APPS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
AUTOSTART_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"

removed=0
for name in aira aira-node aira-desktop; do
  path="$BIN_DIR/$name"
  if [[ -e "$path" ]]; then
    rm -f "$path"
    echo "removed $path"
    removed=1
  fi
done

for name in aira.desktop aira-desktop.desktop; do
  path="$APPS_DIR/$name"
  if [[ -e "$path" ]]; then
    rm -f "$path"
    echo "removed $path"
    removed=1
  fi
done

# Leave autostart alone unless it is ours and user asks via settings; still drop stale hook
# when uninstalling so login does not spawn a missing binary.
if [[ -f "$AUTOSTART_DIR/aira-desktop.desktop" ]]; then
  rm -f "$AUTOSTART_DIR/aira-desktop.desktop"
  echo "removed $AUTOSTART_DIR/aira-desktop.desktop"
  removed=1
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APPS_DIR" >/dev/null 2>&1 || true
fi

if [[ "$removed" -eq 0 ]]; then
  echo "nothing to remove under $PREFIX"
else
  echo "OK: uninstall complete (data under ~/.local/share/aira and ~/.config/aira kept)"
fi
