# Analyze-396 — Installed-product process evidence (P4)

**Status (`#389`):** **HISTORICAL SNAPSHOT** — not proof of current HEAD.  
**Recorded (commit that landed this evidence):** `2026-09-19T07:02:09Z` (author date +0300 → UTC)  
**Tree SHA:** `7bb6716914b069b672a364b9a3990a7a9aed023a`  
**Current SHA-pinned gate:** [`../Analyze-397/`](../Analyze-397/) (`#388`).

Opt-in host evidence from that date. **Not** installed-product complete. **Not** fixture M6. **Not** current HEAD acceptance.

## Ran

- `scripts/installed-product-llm-acceptance.sh` (then defaulting evidence to this Analyze folder)
- Models: `phi:latest`, `llama3.2:latest` (`ollama` on PATH)
- Test: `process::tests::installed_product_two_real_ollama_models` (`#[ignore]`)
- Wall time: 111.81s, result ok
- Artifact: [`run.json`](run.json)

## Proved (as of `7bb6716` only)

- One `ProcessBackend` generated A then B via per-request `host_cli_model` (no process restart)
- `backend=process`, `verification_status` absent, stdout prefixes `Accepted.` / `OK`
- File-weight `model_ref` fail-closed
- 1ms wait returned timeout fail-closed (not VERIFIED)

## Not run

GUI Settings click-through, Compare mid-kill, file Verify→Prepare, failed Stop window, widths 560 / 900 / 1600.
