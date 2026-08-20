# Analyze-102 — DI crystallize (QUEUE #67)

## Interview-complete rationale
QUEUE `#67` + phase-d-plan §6b D5.3 lock Done when / Out.

## Crystallized spec

1. After share ALLOW: materialize signed ModelArtifact + ShareOffer from `models/cache` (activated pointer).
2. Visibility `local`|`opt_in`; default `local`, `allow_download=false`.
3. Event `op:share-published:…`; pointer `models/share-offer.latest.json`.
4. No remote push / capability ad (`#68` Out).
5. CLI `aira models publish --model-ref` performs gate + local publish.
