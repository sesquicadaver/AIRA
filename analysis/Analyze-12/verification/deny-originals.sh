#!/usr/bin/env bash
# Soft immutability gate for AIRA originals.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
dirty="$(git status --porcelain -- 'Manifesto etc' 'Meditation_About' || true)"
if [[ -n "${dirty}" ]]; then
  echo "IMMUTABILITY VIOLATION: originals were modified:" >&2
  echo "${dirty}" >&2
  exit 1
fi
echo "OK: Manifesto etc/ and Meditation_About/ unchanged in working tree"
exit 0
