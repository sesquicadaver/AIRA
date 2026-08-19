# Verification Checklist — Analyze-25

- [x] revoke → sync → UnknownKey on verify
- [x] re-upsert → RevokedKey
- [x] refuse revoke local-test
- [x] list shows REVOKED section
- [x] cargo crypto tests + workspace PASS
- [x] clippy `-D warnings` PASS
- [x] originals unchanged
- [x] docs/crypto.md CRL section
