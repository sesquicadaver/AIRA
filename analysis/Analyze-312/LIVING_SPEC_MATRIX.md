# Living Spec Matrix — Analyze-312 / QUEUE `#277`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| Status refresh без full weights hash | `crates/aira-flow/src/activate_gate.rs` (`ObserveLight` + observe-ready cache) | `observe_second_pass_skips_full_weight_hash` |
| Cached/versioned ready | `models/activated.observe-ready.json` (pointer_fp + len/mtime) | `observe_rehashes_when_*` |
| Start/Stop apply не тим самим full hash кожен tick | той самий `observe()` шлях через `ModelTripleSnapshot::load` | unit + `phase_q_doc` tip |
| Admission не послаблена | `check_activated` → `AdmitFull` | `admit_always_full_hashes_even_after_observe_cache` |
| RFC-0166 | `specs/rfc/AIRA-RFC-0166-model-light-observe.md` | `phase_q_rfc_0166_present` |
