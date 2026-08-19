#!/usr/bin/env bash
# Fail if Analyze-7 reintroduces foreign-workspace markers.
set -euo pipefail
ANALYZE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$ANALYZE_DIR/../.." && pwd)"
cd "$ROOT"
# Assembled so this file is not a false positive for the deny-list itself.
P1='M''IND'
P2='s''TOR-'
P3='stor-to-''book0'
P4='M''IND_ROOT'
P5='\.\./M''IND'
PATTERN="(^|[^A-Za-z0-9_])${P1}([^A-Za-z0-9_]|\$)|${P2}|${P3}|${P4}|${P5}"
hits="$(rg -n -e "$PATTERN" "$ANALYZE_DIR" || true)"
if [[ -n "${hits}" ]]; then
  echo "FOREIGN WORKSPACE REF DETECTED in Analyze-7:" >&2
  echo "${hits}" >&2
  exit 1
fi
echo "OK: Analyze-7 has no foreign-workspace markers"
exit 0
