# AIRA-RFC-0019 — local rating evidence publish (RFC-R)

## 1. Summary

`publish_rating` in CSU `aira:csu:model.rating` materializes a signed contextual `ModelRatingEvidence` CustomArtifact, writes `models/rating.latest.json`, and emits `op:rating-published:…`. `network=none`. No CLI in this atom (`#71`).

## 2. Scope

- `csu/model-rating`
- Pointer `models/rating.latest.json`
- Schema validation against `rating-evidence:0.1`

## 3. Non-Goals

```text
CLI models rate (#71)
upgrade recommendation (D7)
marketplace / global scoreboard / federation
```

## 4. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6c D6.2; QUEUE `#70`.
