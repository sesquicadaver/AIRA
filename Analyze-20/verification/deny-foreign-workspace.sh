#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
test -f crates/aira-object/src/crypto.rs
grep -q 'fn verify_ed25519' crates/aira-object/src/crypto.rs
echo "OK: aira-object crypto present"
exit 0
