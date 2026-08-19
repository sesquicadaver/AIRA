# Living Spec Matrix — Analyze-80

| ТЗ | Модуль | Тести / gate |
|----|--------|----------------|
| aira-core ↛ node/peer/concrete CSU | `scripts/dep_firewall.py` | `--self-test` core→node + transitive CSU; live `cargo metadata` |
| CSU ↛ CSU | same | `--self-test` execution→verification |
| no import cycles | same (normal+build edges) | `--self-test` a↔b |
| CI red on violation | `.github/workflows/ci.yml` step `dependency firewall` | self-test + live check |
