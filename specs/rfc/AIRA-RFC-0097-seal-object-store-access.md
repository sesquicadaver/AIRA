# AIRA-RFC-0097 — Seal object_store_access behind store-backend

## 1. Summary

Phase J `#201`: `aira_object::object_store_access::{mint, storage_token}` are not on the default `aira-object` prelude. They compile only with Cargo feature `store-backend`, enabled solely by `aira-core` ObjectStore backends (not a CSU API). CSUs receive [`Handle`](../../crates/aira-object/src/handle.rs) from `ObjectStore::create` and may read `Handle::object_ref`. Bind checks from RFC-0084 are unchanged.

## 5. Non-Goals

VRA runtime body (`#202`); event-log authority (`#203`); Book II mesh; changing C0 case ids.

## 10. Contract

```text
default aira-object prelude — no object_store_access
feature store-backend — aira-core only
CSU crates — do not enable store-backend; do not import object_store_access
c0.object.handle_opacity — Handle from ObjectStore::create, not mint
```

## 15. Tests

```text
cargo test -p aira-object object_store_access_is_not_in_the_default_prelude
cargo test -p aira-object store_backend_feature_is_only_enabled_by_aira_core
cargo test -p aira-object csu_sources_do_not_import_object_store_access
cargo test -p aira-core handle_
cargo test -p aira-conformance c0
```
