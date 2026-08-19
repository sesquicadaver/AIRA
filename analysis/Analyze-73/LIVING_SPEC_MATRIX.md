# Living Spec Matrix — Analyze-73

| ТЗ | Модуль | Тести / gate |
|----|--------|----------------|
| LS5 / README schema fixtures | `aira schema validate --fixtures` | local 26 passed / 0 failed; CI step `schema fixtures` |
| Conformance C0 | `aira conformance run --profile C0` | local 5/5; CI step `conformance C0`; fail if `failed > 0` |
| Conformance C1 | `aira conformance run --profile C1` | local 4/4; CI step `conformance C1` |
| C2 not in this row | workflow omits C2 | grep: no `--profile C2` in `ci.yml` |
| Out: schema semantics | fixtures unchanged | no `fixtures/` / `schemas/` diff |
