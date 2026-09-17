# Installed-product LLM acceptance (opt-in)

This is **not** the default CI M6 fixture harness.

M6 (`phase_x_m6_*`, RFC-0240) proves staff-path admission with **fixture weights**.
It does **not** prove two real host Ollama models end-to-end.

## Gate (manual / nightly)

Prerequisites: `ollama` on PATH, two pulled models (e.g. `llama3:latest` and another), Desktop built from this tree.

1. Settings → Use Ollama process → select model A → Save → Restart node.
2. Work → Required → model A → submit once (Executed ≠ VERIFIED).
3. Without restart: Required → model B → submit; executed/applied must track B (RFC-0243).
4. Compare A|B; optionally stop/kill one model mid-run and confirm A result retained.
5. File path: Enable local add → Add weights → Verify (artifact path) → Prepare → Work ready.
6. Set `llm_process_timeout_ms` low; confirm timeout error (not malformed HTTP).
7. Quit while Stop fails → window stays; no silent pidfile wipe.
8. Resize to ~560 / 900 / 1600 and confirm Work/Settings usable.

Record command output + screenshots under `analysis/Analyze-N/` when closing the gate.
