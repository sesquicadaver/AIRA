# Getting started

## What is this?

AIRA Desktop is the local window for running tasks on this computer and reading an honest system status. It does not invent a model name, a verified result, or a network state that was never observed.

Help · F1 is always available offline. It uses the topics shipped with this version — no internet and no LLM are required to open Help.

## What to do?

1. Open **Work**.
2. Describe a task in your own words (for example `Calculate 2 + 2`).
3. Press **Run** or **Ctrl+Enter** (⌘+Enter on Mac). Enter alone inserts a new line.
4. If nothing runs, open **System → Program** and press **Start**.

You can open Help any time; Esc closes only Help and never cancels a run or clears your draft.

## What should happen?

You see an answer with run status, check, and origin. Built-in calculation can succeed without a local language model. Provenance stays honest: mock is not verified, and an undefined model is stated explicitly.

## If it failed?

- Empty task text is rejected — type something first.
- If Start fails, read the problem banner and open Help for that code (context Help prefers the last problem topic).
- If the node was stopped, Desktop may start it before Run; if that still fails, use System → Program.

## Related

- help:work.submit — how Run and Ctrl+Enter work
- help:node.lifecycle — Start / Stop the local program
- help:settings.apply — Saved vs Applied settings
