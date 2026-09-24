# Analyze-397 — Installed acceptance on concrete SHA (`#388`)

**Queue:** `#388`  
**Tree under test:** `3dbcd84f92e7d2036c951540b38fbd842d0c01c0` (post `#367`+K + D `#383`–`#387`)  
**Recorded:** 2026-09-24T18:48:25Z  
**Not:** Analyze-396 as proof of HEAD. **Not:** claiming GUI real-Ollama click-through complete.

## Ran

- `scripts/installed-product-llm-acceptance.sh`
- Models: `llama3.2:latest` (`a80c4f17acd5`), `llama3:latest` (`365c0bd3c000`)
- Process: `installed_product_two_real_ollama_models` (`#[ignore]`) — A then B, file-weight fail-closed, 1ms timeout
- Staff path at same SHA: `phase_x_m6_acceptance` (CLI+HTTP) + `phase_x_m6_gui` (GUI fixture readiness)
- Artifact: [`run.json`](run.json)

## Proved

| Surface | Result |
|---------|--------|
| Process real Ollama A→B without restart | ok (`backend=process`, no VERIFIED) |
| Negatives: file-weight on ollama, 1ms timeout | fail-closed |
| CLI+HTTP two-model admission (M6 fixtures) | ok |
| GUI catalog/Work readiness two models (M6 fixtures) | ok |
| SHA + Ollama digests in evidence | present |

## Not run (manual / later)

Real-Ollama GUI Settings click-through, Compare mid-kill, file Verify→Prepare GUI, failed Stop window, widths 560/900/1600.
