# AIRA-RFC-0017 — local capability advertisement (RFC-D/E)

## 1. Summary

On successful local publish, the host also publishes a signed CapabilityDescriptor (`aira:schema:capability:descriptor:0.1`) as a `CustomArtifact`, writes `models/capability-ad.latest.json`, and emits `op:capability-advertised:share:{model_ref}:local`. Capability scope is always `local`. CLI `aira models publish` and `aira models share` share this path. No federation / DHT / marketplace push.

## 2. Problem Statement

D5.4 closes the custom share wave by advertising a host-local capability that references the published ModelArtifact + ShareOffer without remote discovery.

## 3. Scope

- Capability ad inside `publish_local` (`csu/model-acquisition`)
- Pointer `models/capability-ad.latest.json`
- CLI alias `aira models share`
- Event `op:capability-advertised:share:…:local`

## 4. Non-Goals

```text
federation push
DHT / discv advertise
rating (D6)
marketplace
remote DiscoveryRegistry sync
```

## 5. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6b D5.4; QUEUE `#68`.
