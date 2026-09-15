# Read a result

## What is this?

The result area leads with the **answer**, then run status, check, and **origin** (provenance). Technical hashes and raw JSON belong under Details — not as the first thing a person must decode.

## What to do?

1. Read the answer text.
2. Read **Origin**: mock ≠ verified; an undefined or inactive model is stated; local generation is not auto-VERIFIED.
3. Read **Requested / Applied / Executed**: what Work asked for, what was selected/admitted at submit, and what actually ran. For reference mock, Executed is `mock (no model run)` — not the requested model_ref.
4. Open **Details** only when you need technical JSON.

## What should happen?

You get an honest provenance line and model triple for this run. Built-in calculation can show VERIFIED when the execution path truly verified it; text generation without that path must not fake VERIFIED. A noticeable demo banner appears on Work while the staff executor is still reference mock.

## If it failed?

- No answer yet → still waiting (help:work.waiting) or check Recent events.
- Collapse / needs-human states may have no fake answer — follow the message, do not invent one.
- Generation failed for missing model → help:model.unavailable.
- Demo/mock banner → open Settings → Models (or configure a process backend); choice ≠ VERIFIED.

## Related

- help:work.submit — run another task
- help:model.unavailable — when a model is required but missing
- help:model.select — Settings catalog path
- help:start — first-run path
