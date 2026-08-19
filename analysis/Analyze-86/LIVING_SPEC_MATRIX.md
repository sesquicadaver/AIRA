# Living Spec Matrix — Analyze-86

| ТЗ | Модуль | Тести |
|----|--------|-------|
| EVO-2 §3.3 reference-local plane | `docs/operational-plane.md` | operator wording locked |
| Module rustdoc | `crates/aira-flow/src/{lib,plane,local}.rs` | clippy `-D warnings`; existing `aira-flow` lib tests |
| C1/C0 pointer | `crates/aira-conformance/src/{c1,c0}.rs` | `aira-conformance` lib tests |
| Cross-links | `docs/{demo,local-node,conformance,crypto,csu-development}.md`, README | no drain/loop change |
| Drain semantics | `OperationalPlane::drain_from` | comment only; body identical |
