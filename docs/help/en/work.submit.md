# Run a task

## What is this?

**Run** sends your task text to the local AIRA node supervised by Desktop. Only one run is active at a time; the UI never blocks on HTTP inside the paint loop.

## What to do?

1. Keep focus in the Work draft.
2. Use **Enter** for a new line.
3. Press **Run** or **Ctrl+Enter** (⌘+Enter on Mac) when the action is available. If the selected host model still needs a slot, the button is **Prepare and run** instead — one prepare, admit, and start; the default tip does not change.
4. Switch sections or open Help freely — the draft is kept on validation errors and failed runs.

Do not mash Run while a previous run is still active; the second press must not create a duplicate job.

## What should happen?

The Work screen moves to a waiting state, then shows an answer with status, check, and origin. Closing Help does not cancel the run.

## If it failed?

- Empty text → rejected with a clear problem (Help opens on `work.submit`).
- Text generation not ready → bind a host model under Settings → Models, or use **Prepare and run** when the row is fit but unbound, then Run again. Choice is not a verified result. A `:cloud` suffix alone does not mean the model is local.
- Node stopped → Desktop may Start first; if Start fails, follow help:node.lifecycle.
- Model / host LLM missing → see help:model.unavailable; Work requires at least one host process LLM.

## Related

- help:work.waiting — while the run is in progress
- help:work.result — how to read the answer and provenance
- help:model.unavailable — when generate-local is blocked
- help:node.lifecycle — Start the local program
