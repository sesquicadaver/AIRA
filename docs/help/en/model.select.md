# Choosing a model

## What is this?

Local models live in the model / execution layer. They are **not** part of `aira-core`, not a GPU marketplace, and not invented by the Work screen.

## What to do?

- Review model-related options under **Settings → Models** when available.
- Prepare / activate models with the documented model tooling (`aira models` / plane registration) outside this Help article.
- Prefer built-in calculation when you only need deterministic math such as `Calculate 2 + 2`.

## What should happen?

Missing a model does not block tasks that do not need an LLM. When a model is required, Desktop states that clearly instead of inventing a name.

## If it failed?

If generation fails because nothing is activated, open help:model.unavailable. Do not treat a blank model field as a successful local LLM.

## Related

- help:model.unavailable — required model not ready
- help:settings.apply — where settings are saved vs applied
- help:work.submit — running without inventing a model
