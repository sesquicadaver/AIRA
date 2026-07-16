#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
P1='M''IND'
P2='s''TOR-'
P3='stor-to-''book0'
P4='M''IND_ROOT'
P5='\.\./M''IND'
PATTERN="(^|[^A-Za-z0-9_])${P1}([^A-Za-z0-9_]|\$)|${P2}|${P3}|${P4}|${P5}"
hits="$(rg -n -e "$PATTERN" Analyze-16 || true)"
if [[ -n "${hits}" ]]; then
  echo "FOREIGN WORKSPACE REF DETECTED in Analyze-16:" >&2
  echo "${hits}" >&2
  exit 1
fi
echo "OK: Analyze-16 has no foreign-workspace markers"
exit 0
