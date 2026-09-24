# Installed-product LLM acceptance (opt-in)

This is **not** the default CI M6 fixture harness.

M6 (`phase_x_m6_*`, RFC-0240) proves staff-path admission with **fixture weights**.
It does **not** prove two real host Ollama models end-to-end.

## Current SHA evidence (`#388`)

Pinned host run (not Analyze-396 as HEAD proof):

| Field | Value |
|-------|-------|
| Analyze | [`analysis/Analyze-397/`](../analysis/Analyze-397/) |
| `git_sha` | `3dbcd84f92e7d2036c951540b38fbd842d0c01c0` |
| Recorded | `2026-09-24T18:48:25Z` |
| Models / digests | `llama3.2:latest` / `a80c4f17acd5`; `llama3:latest` / `365c0bd3c000` |
| Surfaces | process real Ollama A→B + negatives; M6 fixture CLI+HTTP; M6 fixture GUI readiness |

JSON: [`analysis/Analyze-397/run.json`](../analysis/Analyze-397/run.json). Brief: [`BRIEF.md`](../analysis/Analyze-397/BRIEF.md).

Analyze-396 is a **historical snapshot** (`#389`: explicit date/SHA in evidence headers @ `7bb6716` / `2026-09-19`) — **not** Analyze-396 as HEAD proof. Living SHA-pinned gate: Analyze-397 (`#388`).

## Gate (manual / nightly)

Prerequisites: `ollama` on PATH, two pulled models (e.g. `llama3.2:latest` and `llama3:latest`), Desktop built from this tree.

1. Settings → Use Ollama process → select model A → Save → Restart node (system layout settings path; Work readiness must see the same Process bind without copying into data_root).
2. Work → Required → model A → submit once (Executed ≠ VERIFIED).
3. Without restart: Required → model B → submit; executed/applied must track B (RFC-0243). Tampering only tip JSON `host_ollama_model` must fail-closed (R2).
4. Compare A|B; optionally stop/kill one model mid-run and confirm A result retained.
5. File path: Enable local add → Add weights → Verify (artifact path) → Prepare. The file is **visible** under Local file; on the host Ollama executor it is **not** Work ready for generate (prepared ≠ executable via Ollama / R3 / `#383`). Do not expect a silent tip-argv run and do not add a file-executor only for this check.
6. Set `llm_process_timeout_ms` low; confirm timeout error (not malformed HTTP).
7. Quit while Stop fails → window stays; no silent pidfile wipe.
8. Resize to ~560 / 900 / 1600 and confirm Work/Settings usable.

Record command output + screenshots under `analysis/Analyze-N/` when closing the gate.

## Opt-in process evidence (not GUI click-through)

Default CI does **not** run real `ollama`. On a host that already has two pulled models:

```bash
AIRA_OLLAMA_A=llama3.2:latest AIRA_OLLAMA_B=llama3:latest \
  scripts/installed-product-llm-acceptance.sh
```

That runs `installed_product_two_real_ollama_models` (`#[ignore]`), then M6 fixture CLI/HTTP + GUI readiness at the same `git rev-parse HEAD`, writing SHA + digests into `analysis/Analyze-397/run.json` by default. It covers Required A then B on one ProcessBackend (no restart), file-weight fail-closed, and a 1ms timeout, plus staff-path M6 surfaces. It does **not** cover real-Ollama GUI Settings click-through, Compare mid-kill, failed Stop, or widths 560/900/1600. Workflow: [`.github/workflows/installed-product-llm.yml`](../.github/workflows/installed-product-llm.yml) (`workflow_dispatch` only; no-op if `ollama` is absent). Not a merge gate.
