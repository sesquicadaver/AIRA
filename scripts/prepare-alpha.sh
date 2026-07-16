#!/usr/bin/env bash
# Prepare aira-mvp-v0.1.0-alpha.1 local release pack (Issue #79).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
OUT="${1:-$ROOT/release/alpha}"
mkdir -p "$OUT"/{bin,schemas,fixtures,docs,conformance}

echo "== build =="
cargo build -p aira-cli -p aira-node --release
cp -f target/release/aira target/release/aira-node "$OUT/bin/"

echo "== copy schemas/fixtures/docs =="
cp -a schemas/. "$OUT/schemas/"
cp -a fixtures/. "$OUT/fixtures/"
cp -f docs/demo.md docs/csu-development.md docs/local-node.md docs/conformance.md \
  docs/RELEASE_NOTES-v0.1.0-alpha.1.md "$OUT/docs/" 2>/dev/null || true
cp -f docs/canonical-terminology.md "$OUT/docs/" 2>/dev/null || true

echo "== conformance reports =="
./target/release/aira conformance run --profile C0 --out "$OUT/conformance/c0"
./target/release/aira conformance run --profile C1 --out "$OUT/conformance/c1"

echo "== smoke DoD =="
SMOKE="$OUT/smoke.aira"
rm -rf "$SMOKE"
./target/release/aira --root "$SMOKE" init
./target/release/aira --root "$SMOKE" identity create --name alpha
./target/release/aira --root "$SMOKE" problem submit --text "Calculate 2 + 2" | tee "$OUT/smoke-2plus2.txt"

cat > "$OUT/MANIFEST.txt" <<EOF
aira-mvp-v0.1.0-alpha.1
built: $(date -u +%Y-%m-%dT%H:%M:%SZ)
binaries: aira, aira-node
schemas: included
fixtures: included
conformance: C0+C1 reports under conformance/
EOF

echo "OK: release pack at $OUT"
