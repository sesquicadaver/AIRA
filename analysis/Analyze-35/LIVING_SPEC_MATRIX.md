# Living Spec Matrix — Analyze-35

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Hello v1 + signed X25519 | `handshake` | roundtrip / multi-dial |
| Noise XX after hello | `noise` | envelope roundtrip encrypted |
| Remote static == hello | `session` finish_* / `ensure_noise_static_bind` | `noise_static_bind_rejects_mismatch` |
| `local.x25519` mode 0600 | `noise::load_or_create_noise_static` | `noise_static_file_created_mode_600` |
| Encrypted envelopes | `AuthenticatedPeer` | `trusted_peers_hello_and_envelope_roundtrip` |
| Cleartext post-Noise rejected | `recv_envelope` | `envelope_issuer_mismatch_rejected` |
| Daemon multi dial | listen loop | `listen_accepts_multiple_hello_only_dials` |
