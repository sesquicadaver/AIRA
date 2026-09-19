# Analyze-396 — Installed-product process evidence (P4)

Opt-in host evidence. **Not** installed-product complete. **Not** fixture M6.

## Ran

- `scripts/installed-product-llm-acceptance.sh`
- Models: `phi:latest`, `llama3.2:latest` (`ollama` on PATH)
- Test: `process::tests::installed_product_two_real_ollama_models` (`#[ignore]`)
- Wall time: 111.81s, result ok
- Artifact: [`run.json`](run.json)

## Proved

- One `ProcessBackend` generated A then B via per-request `host_cli_model` (no process restart)
- `backend=process`, `verification_status` absent, stdout prefixes `Accepted.` / `OK`
- File-weight `model_ref` fail-closed
- 1ms wait returned timeout fail-closed (not VERIFIED)

## Not run

GUI Settings click-through, Compare mid-kill, file Verify→Prepare, failed Stop window, widths 560 / 900 / 1600.
