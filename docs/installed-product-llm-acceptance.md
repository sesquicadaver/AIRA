# Installed-product LLM acceptance (opt-in)

This is **not** the default CI M6 fixture harness.

M6 (`phase_x_m6_*`, RFC-0240) proves staff-path admission with **fixture weights**.
It does **not** prove two real host Ollama models end-to-end.

## Gate (manual / nightly)

Prerequisites: `ollama` on PATH, two pulled models (e.g. `llama3:latest` and another), Desktop built from this tree.

1. Settings → Use Ollama process → select model A → Save → Restart node (system layout settings path; Work readiness must see the same Process bind without copying into data_root).
2. Work → Required → model A → submit once (Executed ≠ VERIFIED).
3. Without restart: Required → model B → submit; executed/applied must track B (RFC-0243). Tampering only tip JSON `host_ollama_model` must fail-closed (R2).
4. Compare A|B; optionally stop/kill one model mid-run and confirm A result retained.
5. File path: Enable local add → Add weights → Verify (artifact path) → Prepare. The file is **visible** under Local file; on the host Ollama executor it is **not** Work ready for generate (prepared ≠ executable via Ollama / R3 / `#383`). Do not expect a silent tip-argv run and do not add a file-executor only for this check.
6. Set `llm_process_timeout_ms` low; confirm timeout error (not malformed HTTP).
7. Quit while Stop fails → window stays; no silent pidfile wipe.
8. Resize to ~560 / 900 / 1600 and confirm Work/Settings usable.

Record command output + screenshots under `analysis/Analyze-N/` when closing the gate.

## Opt-in process evidence (not GUI)

Default CI does **not** run real `ollama`. On a host that already has two pulled models:

```bash
AIRA_OLLAMA_A=phi:latest AIRA_OLLAMA_B=llama3.2:latest \
  scripts/installed-product-llm-acceptance.sh
```

That runs `installed_product_two_real_ollama_models` (`#[ignore]`). It covers Required A then B on one ProcessBackend (no restart), file-weight fail-closed, and a 1ms timeout. It does **not** cover GUI click-through, Compare mid-kill, failed Stop, or widths 560/900/1600. Workflow: [`.github/workflows/installed-product-llm.yml`](../.github/workflows/installed-product-llm.yml) (`workflow_dispatch` only; no-op if `ollama` is absent). Not a merge gate.

