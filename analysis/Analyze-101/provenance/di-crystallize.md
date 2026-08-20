# Analyze-101 — DI crystallize (QUEUE #66)

## Interview-complete rationale
QUEUE `#66` + phase-d-plan §6b D5.2 lock Done when / Out.

## Crystallized spec

1. `request_publish`: no policy / `share_custom_models=false` → DENY + Event; `true` → ALLOW.
2. ALLOW does **not** create ShareOffer / publish bytes (`#67` Out).
3. CLI: `aira models publish --model-ref`; `policy set --share-custom-models`.
4. Reuse acquisition policy file; separate share decision pointer.
5. Out: local publish, capability ad, rating, network.
