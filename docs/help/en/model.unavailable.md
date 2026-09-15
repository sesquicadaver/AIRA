# Model unavailable

## What is this?

A task asked for local text generation, but no suitable model is **ready** or **selected** for generate. This is a fail-closed signal — not a silent fallback to a made-up model, and not the same problem as “AIRA is stopped.”

Help for this topic is **offline** (embedded; no network / no LLM to read it).

## What to do?

1. If the task allows it, use built-in calculation (math does not need a model; Compare also cannot run on math alone).
2. Open **Settings → Models** (not System → Model — System only observes):
   - **Scan** the catalog, or **Enable local add** → **Add file…**
   - **Select** Auto or a specific available row
   - **Prepare** until ready; read the ready reason
3. On **Work**, set executor to **Auto**, **Specific model**, or **Compare** (two different ready models), then **Run** again.
4. If Compare is blocked, fix the unready leg (A or B) — the UI will not silently use the tip instead.
5. If you only changed prefs that need apply/restart, see help:settings.apply / help:node.lifecycle.

Optional CLI tooling exists (`aira models …`) but is **not** required when the Settings catalog buttons are available.

## What should happen?

Deterministic execution can still succeed without an LLM. Generate stays unavailable until a real prepared/selected backend exists. selected ≠ ready ≠ used stays honest after a successful path. Compare never silently substitutes an unready model.

## If it failed?

- Do not expect VERIFIED free-form text without a ready backend.
- Demo / reference mock on Work is not “model available” for real generation — help:work.result and Settings → Models.
- If **Run** cannot Start node, see help:node.lifecycle first.

## Related

- help:model.select — choosing / preparing a local model in Settings
- help:work.submit — retry the task
- help:work.result — provenance when generation runs (or mock)
- help:node.lifecycle — when the node itself will not start
