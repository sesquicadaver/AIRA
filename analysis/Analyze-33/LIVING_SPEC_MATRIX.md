# Living Spec Matrix — Analyze-33

| ТЗ / вимога | Модуль | Тести / evidence |
|-------------|--------|------------------|
| peer add trust-gated | `PeerCommands::Add` | CLI smoke: untrusted rejected |
| peer list | `PeerCommands::List` | CLI smoke |
| peer listen recv | `PeerCommands::Listen` | CLI smoke roundtrip |
| peer dial hello | `PeerCommands::Dial` | CLI (API covered by send) |
| peer send envelope | `PeerCommands::Send` | CLI smoke listen receives peer.ping |
| docs | `docs/peer-link.md` | checklist |
