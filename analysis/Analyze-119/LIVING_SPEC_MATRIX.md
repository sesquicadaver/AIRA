# Living Spec — Analyze-119 / #84

| ТЗ | Модуль | Тести |
|----|--------|-------|
| QR encode PNG | `invite_qr.rs` + `invite-export-qr` | `invite_qr` |
| QR decode → invite | `decode_invite_png` | encode_decode_roundtrip_bytes |
| Import trust+book | `import_invite_qr_file` | qr_png_export_import_trust_and_book |
| Reject blank / bad QR | no QR / bad JSON | unit + reject_non_image_file |
| RFC | RFC-0033 | — |
