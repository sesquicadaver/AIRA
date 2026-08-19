# Living Spec Matrix — Analyze-81

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Mechanical split CLI | `commands/{schema,identity,csu,problem,peer,trust,tenant}` + node/federation/conformance | `cargo test -p aira-cli` |
| Clap XOR secret flags | `cli.rs` + `main.rs` clap tests | `register_xor_secret_flags` / `rotate_xor_secret_flags` |
| Tenant seed IO unchanged | `tenant_secret.rs` (not moved) | tenant_secret tests |
| No new subcommands | clap enums extracted, not extended | `--help` surface |
