# Living Spec Matrix — Analyze-32

| ТЗ / вимога | Модуль | Тести |
|-------------|--------|--------|
| Mutual hello + envelope | `aira-peer::session` | `trusted_peers_hello_and_envelope_roundtrip` |
| Untrusted reject | handshake + trust | `untrusted_peer_rejected_at_handshake` |
| Revoked cannot dial | `dial` | `revoked_peer_cannot_dial` |
| Issuer bind | `recv_envelope` / `send_envelope` | `envelope_issuer_mismatch_rejected` |
| Frame max | `frame` | `frame_too_large_rejected` |
| Docs | `docs/peer-link.md` | checklist |
