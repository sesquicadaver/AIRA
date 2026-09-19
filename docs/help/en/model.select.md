# Choosing a model

## What is this?

Local models belong to the model / execution layer — **not** `aira-core`, not a GPU marketplace, and not invented by the Work screen. Help works **offline** (embedded articles; no network and no LLM required to read this topic).

**Settings ≠ System:** change the catalog only under **Settings → Models**. **System → Model** shows **selected**, **ready**, and **used in last result** as observed facts — it is not a second catalog editor. There is one catalog with two sources: **Host Ollama** and **Local file**. Each source shows only the actions it allows.

## What to do?

Canonical Desktop path (no CLI required):

1. Open **Settings → Models** and pick a **source**.
2. **Host Ollama:** refresh the host list, pick a model (that choice does not change the default), then **Make default** or **Use Ollama process**. Make default writes the default tip (signed bind marker; not weight-verified) and sets process for the **next** node start. Until you **Stop** then **Start**, the running executor may still be mock. If the host list is still loading, Use Ollama finishes the bind after the list returns.
3. **Local file:** note **Model storage** (observe-only). **Scan**, or **Enable local add** → **Add file…**. The **artifact path** for **Verify** is on this add-file step, not under Technical details. **Select** a row (not verified). **Prepare** when the row is a local file and not ready. Host Ollama rows are not listed here and have no Prepare.
4. On **Work**, choose **Auto**, **Specific model**, or **Compare**. Work requires a host process LLM. Built-in math demo is **legacy**, not a model-free product path. Compare needs two different available models; if B fails, A stays — no silent substitute.
5. If Work is blocked for missing host LLM, bind Ollama under Settings → Models and restart. Mock is not product Work.

Optional tooling (not required for the GUI path): `aira models scan|list|activate`; env `AIRA_LLM_BACKEND=process` for Dev.

## What should happen?

- Missing a model / host LLM **blocks** Work generate (RFC-0242); there is no math-without-model product escape.
- Desktop keeps **selected ≠ ready ≠ used** honest — never invent a model name or copy selected into used.
- Choice / Prepare / Select never silently mean VERIFIED free-form text.
- Compare never silently swaps an unready model for the tip or another catalog row.

## If it failed?

- Generation blocked because nothing is ready → follow help:model.unavailable (same Settings → Models steps).
- Compare blocked because A or B is empty, identical, or unready → fix that leg in Settings → Models; do not expect Auto tip to fill the gap.
- Blank or “no model selected” is not a successful local LLM.
- If the node is Running, the failure is about model capability — not “AIRA is stopped” (help:node.lifecycle only if Start node itself failed).

## Related

- help:model.unavailable — required model not ready
- help:settings.apply — where settings are saved vs applied
- help:work.submit — running without inventing a model
- help:work.result — mock banner and requested / applied / executed
