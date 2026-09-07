# Model unavailable

## What is this?

A task asked for local text generation, but no suitable model backend is ready or selected. This is a fail-closed signal — not a silent fallback to a made-up model.

## What to do?

1. Prefer built-in calculation when the task allows it.
2. Activate / register a local model through the model layer, then retry.
3. Confirm Settings were applied if you just changed model-related options (help:settings.apply).

## What should happen?

Deterministic execution can still succeed without an LLM. Generation stays unavailable until a real backend is present.

## If it failed?

Do not expect VERIFIED free-form text without an activated backend. If Run itself cannot start the node, see help:node.lifecycle. For choosing models, see help:model.select.

## Related

- help:model.select — choosing / preparing a local model
- help:work.submit — retry the task
- help:work.result — how provenance is shown when generation runs
