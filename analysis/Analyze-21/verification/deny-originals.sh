#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
dirty="$(git status --porcelain -- 'Manifesto etc' 'Meditation_About' || true)"
if [[ -n "${dirty}" ]]; then
  echo "IMMUTABILITY VIOLATION:" >&2
  echo "${dirty}" >&2
  exit 1
fi
echo "OK: originals unchanged"
exit 0
