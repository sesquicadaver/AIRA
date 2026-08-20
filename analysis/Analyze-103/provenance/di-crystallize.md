# Analyze-103 — DI crystallize (QUEUE #68)

## Interview-complete rationale
QUEUE `#68` + phase-d-plan §6b D5.4 lock Done when / Out. Prior `#67` already materializes ShareOffer; this atom adds local-only capability advertisement alongside publish.

## Crystallized spec

1. On successful `publish_local`: also publish signed CapabilityDescriptor (`aira:schema:capability:descriptor:0.1`) as CustomArtifact.
2. Capability `scope.scope_type` **must** be `local` (even if ShareOffer visibility is `opt_in`).
3. Pointer `models/capability-ad.latest.json`; Event `op:capability-advertised:…`.
4. CLI: `aira models publish` and `aira models share` (alias) both run the same local path.
5. No DiscoveryRegistry federation/DHT push; `network=none`.
