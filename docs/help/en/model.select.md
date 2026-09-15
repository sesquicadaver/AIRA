# Choosing a model

## What is this?

Local models belong to the model / execution layer — **not** `aira-core`, not a GPU marketplace, and not invented by the Work screen. Help works **offline** (embedded articles; no network and no LLM required to read this topic).

**Settings ≠ System:** change the catalog only under **Settings → Models**. **System → Model** shows **selected**, **ready**, and **used in last result** as observed facts — it is not a second catalog editor.

## What to do?

Canonical Desktop path (no CLI required):

1. Open **Settings → Models**.
2. **Scan** the local lifecycle catalog (or **Enable local add** → **Add file…** for a local weight file).
3. **Select** Auto or a specific row. Selection is **not** VERIFIED.
4. **Prepare** when the row is not ready yet; read the ready reason on the catalog row.
5. On **Work**, choose executor **Auto** or **Specific model**, then **Run**. Built-in calculation (for example `Calculate 2 + 2`) does **not** need a model.
6. If Work shows the **demo / reference mock** banner, open **Settings → Models** — mock text generation is not a real local LLM run (see help:work.result).

Optional tooling (not required for the GUI path): `aira models scan|list|activate`.

## What should happen?

- Missing a model does **not** block tasks that do not need an LLM (math readiness ≠ generate readiness).
- Desktop keeps **selected ≠ ready ≠ used** honest — never invent a model name or copy selected into used.
- Choice / Prepare / Select never silently mean VERIFIED free-form text.

## If it failed?

- Generation blocked because nothing is ready → follow help:model.unavailable (same Settings → Models steps).
- Blank or “no model selected” is not a successful local LLM.
- If the node is Running, the failure is about model capability — not “AIRA is stopped” (help:node.lifecycle only if Start node itself failed).
- Do **not** look for a Work **Compare** mode in this Help topic — that path is separate.

## Related

- help:model.unavailable — required model not ready
- help:settings.apply — where settings are saved vs applied
- help:work.submit — running without inventing a model
- help:work.result — mock banner and requested / applied / executed
