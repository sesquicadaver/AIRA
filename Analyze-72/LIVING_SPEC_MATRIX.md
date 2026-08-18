# Living Spec — Analyze-72 / QUEUE #37

| ТЗ | Модуль | Тести |
|----|--------|-------|
| `--secret-hex-file PATH` | CLI register **and** rotate | persist pubkey matches seed |
| Oversize (>4KiB / 4097th byte) | `load_from_reader` | err, no prefix parse |
| `PATH=-` stdin | `load_from_reader` | non-TTY ok; TTY fail-closed |
| Hex normalize B1 | `parse_seed_hex` | newline ok; inner space/`0x`/raw fail |
| XOR `--secret-hex` | clap `try_parse_from` | Err on register **and** rotate |
| Never print secret | CLI errors | message has path/flag, not seed |

**Honest:** CLI import only. `--secret-hex` remains demo. Not node identity secret-file.
