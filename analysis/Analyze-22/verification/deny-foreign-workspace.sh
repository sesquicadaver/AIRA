#!/usr/bin/env bash
set -euo pipefail
ANALYZE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$ANALYZE_DIR/../.." && pwd)"
cd "$ROOT"
grep -q 'fn set_primary_signer' crates/aira-object/src/crypto.rs
grep -q 'register_node_identity' crates/aira-flow/src/local.rs
echo "OK: primary signer + LocalSession wiring present"
exit 0
