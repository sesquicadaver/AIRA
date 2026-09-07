# Read a result

## What is this?

The result area leads with the **answer**, then run status, check, and **origin** (provenance). Technical hashes and raw JSON belong under Details — not as the first thing a person must decode.

## What to do?

1. Read the answer text.
2. Read **Origin**: mock ≠ verified; an undefined or inactive model is stated; local generation is not auto-VERIFIED.
3. Open **Details** only when you need technical JSON.

## What should happen?

You get an honest provenance line for this run. Built-in calculation can show VERIFIED when the execution path truly verified it; text generation without that path must not fake VERIFIED.

## If it failed?

- No answer yet → still waiting (help:work.waiting) or check Recent events.
- Collapse / needs-human states may have no fake answer — follow the message, do not invent one.
- Generation failed for missing model → help:model.unavailable.

## Related

- help:work.submit — run another task
- help:model.unavailable — when a model is required but missing
- help:start — first-run path
