#!/usr/bin/env bash
# Soft gate: workspace must remain the AIRA checkout (not a foreign clone).
set -euo pipefail
ANALYZE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROOT="$(cd "$ANALYZE_DIR/../.." && pwd)"
cd "$ROOT"
test -f Cargo.toml
test -d crates/aira-node
grep -q 'name = "aira-node"' crates/aira-node/Cargo.toml
echo "OK: foreign-workspace deny (aira-node present)"
exit 0
