# Model unavailable

## What is this?

A task asked for local text generation, but no suitable model is **ready** or **selected** for generate. This is a fail-closed signal — not a silent fallback to a made-up model, and not the same problem as “AIRA is stopped.”

Help for this topic is **offline** (embedded; no network / no LLM to read it).

## What to do?

1. If the task allows it, use built-in calculation (math does not need a model).
2. Open **Settings → Models** (not System → Model — System only observes):
   - **Scan** the catalog, or **Enable local add** → **Add file…**
   - **Select** Auto or a specific available row
   - **Prepare** until ready; read the ready reason
3. On **Work**, set executor to **Auto** or **Specific model** that is ready, then **Run** again.
4. If you only changed prefs that need apply/restart, see help:settings.apply / help:node.lifecycle.

Optional CLI tooling exists (`aira models …`) but is **not** required when the Settings catalog buttons are available.

## What should happen?

Deterministic execution can still succeed without an LLM. Generate stays unavailable until a real prepared/selected backend exists. selected ≠ ready ≠ used stays honest after a successful path.

## If it failed?

- Do not expect VERIFIED free-form text without a ready backend.
- Demo / reference mock on Work is not “model available” for real generation — help:work.result and Settings → Models.
- If **Run** cannot Start node, see help:node.lifecycle first.
- Do **not** follow instructions for a Work **Compare** mode here — that is a separate topic.

## Related

- help:model.select — choosing / preparing a local model in Settings
- help:work.submit — retry the task
- help:work.result — provenance when generation runs (or mock)
- help:node.lifecycle — when the node itself will not start
