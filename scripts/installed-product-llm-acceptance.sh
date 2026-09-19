#!/usr/bin/env bash
# Opt-in installed-product LLM evidence. Not default CI.
# Requires ollama on PATH and two pulled models.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export AIRA_INSTALLED_PRODUCT=1
export AIRA_OLLAMA_A="${AIRA_OLLAMA_A:-phi:latest}"
export AIRA_OLLAMA_B="${AIRA_OLLAMA_B:-llama3.2:latest}"
export AIRA_INSTALLED_EVIDENCE="${AIRA_INSTALLED_EVIDENCE:-$ROOT/analysis/Analyze-396/run.json}"

command -v ollama >/dev/null
mkdir -p "$(dirname "$AIRA_INSTALLED_EVIDENCE")"

echo "models: $AIRA_OLLAMA_A | $AIRA_OLLAMA_B"
echo "evidence: $AIRA_INSTALLED_EVIDENCE"

cargo test -p aira-csu-execution-llm --lib installed_product_two_real_ollama_models -- --ignored --nocapture
