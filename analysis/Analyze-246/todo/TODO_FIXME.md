# TODO_FIXME — Analyze-246 (CSU + mock)

- [x] `#211` `execution-llm` MockBackend
- [ ] `#212` Reduction bind (non-math → generate; C1 2+2 stays math.eval.safe)
- [ ] `#213` Plane register (do not dispatch from OperationalPlane in this atom)
- [ ] `#214` Activate gate — placeholder only (`activate_gate_placeholder`); do not treat missing model as VERIFIED
- [ ] `#215` Process backend (ollama/llama.cpp CLI); keep mock for CI
- [ ] Core must stay inference-free (firewall + review)
- [ ] RFC-0104 stays file-free until `#216`
- [ ] Do not mark LLM Execution CSU fully DONE until `#213` lands
