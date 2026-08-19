#!/usr/bin/env bash
# Fail if Manifesto / Meditation trees appear in staged or working tree diffs.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
if git diff --name-only HEAD | grep -E '^(Manifesto etc/|Meditation_About/)' >/dev/null; then
  echo "deny-originals: forbidden path in working tree" >&2
  exit 1
fi
echo "deny-originals: OK"
