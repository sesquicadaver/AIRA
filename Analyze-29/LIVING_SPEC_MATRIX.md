# Living Spec Matrix — Analyze-29

| ТЗ | Модуль | Тести |
|----|--------|-------|
| signature_for (no fallback) | `aira-object` | support publisher test |
| make_*_as / apply_publisher | `aira-csu::support` | `publisher_override_signs_distinct_from_primary` |
| Basic CSUs emit as publisher | `csu/*-basic` | existing CSU suites + compile |
| with_publisher builder | six basic CSUs | same |
| Docs | `docs/crypto.md` | — |
| Immutability | verify-gates | `deny-originals.sh` |
