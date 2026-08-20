# Living Specification — Analyze-96 / QUEUE #61

| ТЗ / атомарний scope | Модуль | Тести |
|----------------------|--------|-------|
| RFC ALLOW gate | `specs/rfc/AIRA-RFC-0010-acquisition-policy-allow.md` | docs |
| auto_download=true → ALLOW + Event | `csu/model-acquisition` `request_download` | `allow_when_auto_download_true_no_transfer` |
| no policy → DENY | same | `deny_without_policy_emits_decision` |
| auto_download=false → DENY | same | `deny_when_auto_download_false` |
| ALLOW ≠ transfer | no quarantine/weights | unit asserts |
| CLI exit 0 / 2 | `crates/aira-cli/.../models.rs` | manual / UltraQA |
