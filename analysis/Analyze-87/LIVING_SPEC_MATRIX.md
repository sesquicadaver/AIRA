# Living Spec Matrix — Analyze-87

| ТЗ | Модуль | Тести |
|----|--------|-------|
| EVO-2 §4.2 status map | `docs/implementation-status.md` | columns Requirement / Source / Implemented in / Tested by / Status |
| Book 0–IV coverage | same | sections Book 0, I, II, III, IV |
| Schema Pack | same + `fixtures/manifest.json` | per-`$id` fixture row |
| C0/C1 | `aira-conformance` `c0.rs` / `c1.rs` | named case ids listed |
| CSU basic | `csu/*-basic` | crate + `c1.csu.manifests` |
| HTTP/peer | `aira-node` / `aira-peer` | labelled POST-MVP |
| No new runtime | git diff | docs + analysis + README/docs pointers only |
