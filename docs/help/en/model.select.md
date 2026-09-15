# Choosing a model

## What is this?

Local models belong to the model / execution layer — **not** `aira-core`, not a GPU marketplace, and not invented by the Work screen. Help works **offline** (embedded articles; no network and no LLM required to read this topic).

**Settings ≠ System:** change the catalog only under **Settings → Models**. **System → Model** shows **selected**, **ready**, and **used in last result** as observed facts — it is not a second catalog editor.

## What to do?

Canonical Desktop path (no CLI required):

1. Open **Settings → Models**.
2. Note the **Model storage** block: storage root under this install’s data root, used space, and free space on the volume (or **unknown** if not observable). This is observe-only — not a marketplace and not edited on System.
3. **Scan** the local lifecycle catalog (or **Enable local add** → **Add file…** for a local weight file).
4. **Select** Auto or a specific row. Selection is **not** VERIFIED.
5. **Prepare** when the row is not ready yet; read the ready reason on the catalog row.
6. On **Work**, choose executor **Auto**, **Specific model**, or **Compare**. Built-in calculation (for example `Calculate 2 + 2`) does **not** need a model and cannot use Compare.
7. **Compare** needs two **different** available catalog models (A and B). Both must be ready before Run. A and B run sequentially on the same draft; if B fails, A’s result stays — there is **no silent substitute** of tip or another model.
8. If Work shows the **demo / reference mock** banner, open **Settings → Models** — mock text generation is not a real local LLM run (see help:work.result).

Optional tooling (not required for the GUI path): `aira models scan|list|activate`.

## What should happen?

- Missing a model does **not** block tasks that do not need an LLM (math readiness ≠ generate readiness).
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
