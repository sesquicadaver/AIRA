# Run a task

## What is this?

**Run** sends your task text to the local AIRA node supervised by Desktop. Only one run is active at a time; the UI never blocks on HTTP inside the paint loop.

## What to do?

1. Keep focus in the Work draft.
2. Use **Enter** for a new line.
3. Press **Run** or **Ctrl+Enter** (⌘+Enter on Mac) when the action is available.
4. Switch sections or open Help freely — the draft is kept on validation errors and failed runs.

Do not mash Run while a previous run is still active; the second press must not create a duplicate job.

## What should happen?

The Work screen moves to a waiting state, then shows an answer with status, check, and origin. Closing Help does not cancel the run.

## If it failed?

- Empty text → rejected with a clear problem (Help opens on `work.submit`).
- Node stopped → Desktop may Start first; if Start fails, follow help:node.lifecycle.
- Model missing for generation → see help:model.unavailable; calculation like `Calculate 2 + 2` can still work without an LLM.

## Related

- help:work.waiting — while the run is in progress
- help:work.result — how to read the answer and provenance
- help:node.lifecycle — Start the local program
