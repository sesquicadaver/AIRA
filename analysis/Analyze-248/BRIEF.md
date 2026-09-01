# BRIEF — Analyze-248

Register `execution-llm` (MockBackend) on OperationalPlane after `#212` DONE so `submit_problem` for a non-math prompt reaches generate-local and completes with CapsuleCompleted. `Calculate 2 + 2` stays `execution-basic` / `math.eval.safe`. Do not implement activate (`#214`), process backend (`#215`), or file RFC-0104.
