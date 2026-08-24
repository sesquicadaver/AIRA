# AIRA-RFC-0065 — Model acquisition mechanical split

## 1. Summary

Phase F `#116`: mechanical split of `csu/model-acquisition/src/lib.rs` into focused modules (`error`, `types`, `manifest`, `policy`, `publish`, `quarantine`, `verify`, `activate`, `util`) with unchanged public API and behavior.

## 5. Non-Goals

Semantic refactor; CLI/API/schema changes; new acquisition features.

## 15. Tests

`cargo test -p aira-csu-model-acquisition`
