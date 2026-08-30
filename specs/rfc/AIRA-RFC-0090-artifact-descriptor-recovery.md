# AIRA-RFC-0090 — Per-artifact-id descriptor recovery

## 1. Summary

Phase I `#192`: `CasArtifactStore` stores a descriptor file per `artifact_id`. The CAS blob remains keyed by content hash. A second descriptor with the same payload is recoverable after `index.json` is lost.

## 5. Non-Goals

Runtime Clock (`#193`); SQLite artifact metadata; removing the legacy first-writer `{hash}.json` sidecar (C0 / RFC-0062).

## 10. Contract

```text
publish(id, bytes) → sha256/.../{hash}.bin (shared) + descriptors/{hex(id)}.json (always)
second publish same bytes, different id → both resolve
delete index.json → reopen merges descriptors/ → both ids resolve
```

## 15. Tests

```text
cargo test -p aira-artifact --lib second_descriptor_same_content_hash_recoverable
```
