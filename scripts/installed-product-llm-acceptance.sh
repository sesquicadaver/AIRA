#!/usr/bin/env bash
# Opt-in installed-product LLM evidence for a concrete git SHA (#388).
# Not default CI. Does not treat Analyze-396 as proof of HEAD.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export AIRA_INSTALLED_PRODUCT=1
export AIRA_OLLAMA_A="${AIRA_OLLAMA_A:-llama3.2:latest}"
export AIRA_OLLAMA_B="${AIRA_OLLAMA_B:-llama3:latest}"
export AIRA_GIT_SHA="${AIRA_GIT_SHA:-$(git rev-parse HEAD)}"
export AIRA_RECORDED_AT="${AIRA_RECORDED_AT:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}"
export AIRA_INSTALLED_EVIDENCE="${AIRA_INSTALLED_EVIDENCE:-$ROOT/analysis/Analyze-397/run.json}"

command -v ollama >/dev/null
mkdir -p "$(dirname "$AIRA_INSTALLED_EVIDENCE")"

digest_for() {
  local name="$1"
  # `ollama list` id column (short digest for the tag).
  ollama list 2>/dev/null | awk -v n="$name" 'NR>1 && $1==n { print $2; exit }'
}

export AIRA_OLLAMA_DIGEST_A="${AIRA_OLLAMA_DIGEST_A:-$(digest_for "$AIRA_OLLAMA_A")}"
export AIRA_OLLAMA_DIGEST_B="${AIRA_OLLAMA_DIGEST_B:-$(digest_for "$AIRA_OLLAMA_B")}"

echo "sha: $AIRA_GIT_SHA"
echo "models: $AIRA_OLLAMA_A ($AIRA_OLLAMA_DIGEST_A) | $AIRA_OLLAMA_B ($AIRA_OLLAMA_DIGEST_B)"
echo "evidence: $AIRA_INSTALLED_EVIDENCE"

cargo test -p aira-csu-execution-llm --lib installed_product_two_real_ollama_models -- --ignored --nocapture

# Staff-path GUI+CLI+HTTP at the same SHA (fixture weights; not real ollama GUI).
echo "running M6 fixture GUI+CLI+HTTP at $AIRA_GIT_SHA"
cargo test -p aira-flow --test phase_x_m6_acceptance -- --nocapture
cargo test -p aira-desktop-runtime --test phase_x_m6_gui -- --nocapture

python3 - <<'PY'
import json, os
from pathlib import Path
path = Path(os.environ["AIRA_INSTALLED_EVIDENCE"])
doc = json.loads(path.read_text())
doc["queue_atom"] = "#388"
doc["git_sha"] = os.environ["AIRA_GIT_SHA"]
doc["recorded_at"] = os.environ["AIRA_RECORDED_AT"]
doc["not_analyze_396_head_proof"] = True
doc["executed"] = dict(doc.get("executed") or {})
doc["executed"]["m6_fixture_cli_http"] = True
doc["executed"]["m6_fixture_gui"] = True
doc["surfaces"] = {
    "process_real_ollama": True,
    "cli_http_fixture_m6": True,
    "gui_fixture_m6": True,
    "gui_real_ollama_clickthrough": False,
}
path.write_text(json.dumps(doc, indent=2) + "\n")
print("enriched", path)
PY
