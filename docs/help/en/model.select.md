# Choosing a model

## What is this?

Local models live in the model / execution layer. They are **not** part of `aira-core`, not a GPU marketplace, and not invented by the Work screen.

## What to do?

- Under **Settings → Models**, select/activate a **local** discovered compatible model when you need an LLM (RFC-0159 amendment 2026-09-10 — anti-usurpation of a single model).
- CLI `aira models scan|list|activate` remains valid tooling.
- Prefer built-in calculation when you only need deterministic math such as `Calculate 2 + 2`.

## What should happen?

Missing a model does not block tasks that do not need an LLM. Desktop shows **selected**, **ready**, and **used in last result** separately — never inventing a name or copying selected into used. Choosing a model is **not** VERIFIED.

## If it failed?

If generation fails because nothing is activated, open help:model.unavailable. Do not treat a blank model field as a successful local LLM. If the node is Running, the failure is about the model Capability — not «AIRA is stopped».

## Related

- help:model.unavailable — required model not ready
- help:settings.apply — where settings are saved vs applied
- help:work.submit — running without inventing a model
