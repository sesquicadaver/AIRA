# Model unavailable

## What is this?

A task asked for local text generation, but no suitable model is **ready** or **selected** for generate. This is a fail-closed signal — not a silent fallback to a made-up model, and not the same problem as “AIRA is stopped.”

Help for this topic is **offline** (embedded; no network / no LLM to read it).

## What to do?

1. Open **Settings → Models** (not System → Model — System only observes):
   - Refresh the host list, pick a model, then **Make default**
   - On **Work**, if the row is fit but unbound, use **Prepare and run** (does not rewrite the default tip)
   - **Scan**, or add a local file and **Prepare** when that file can run
2. On **Work**, leave the selector on **Default: name**, or pick another model. **Compare** opens a second selector. Then **Run** again.
3. If Compare is blocked, fix the unready leg — the UI will not silently use another model.
4. If you only changed settings that need a restart, see help:settings.apply / help:node.lifecycle.

Optional CLI tooling exists (`aira models …`) but is **not** required when the Settings catalog buttons are available. A `:cloud` name suffix is not proof of local execution.

## What should happen?

Generate stays unavailable until a real model is selected. selected ≠ ready ≠ used stays honest after a successful path. Compare never silently substitutes an unready model.

## If it failed?

- Do not expect VERIFIED free-form text without a ready backend.
- Demo / reference mock on Work is not “model available” for real generation — help:work.result and Settings → Models.
- If **Run** cannot Start node, see help:node.lifecycle first.

## Related

- help:model.select — choosing / preparing a local model in Settings
- help:work.submit — retry the task
- help:work.result — provenance when generation runs (or mock)
- help:node.lifecycle — when the node itself will not start
