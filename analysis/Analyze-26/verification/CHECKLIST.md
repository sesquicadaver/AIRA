# Verification Checklist — Analyze-26

- [x] unrevoke clears CRL; upsert then succeeds
- [x] unrevoke alone → verify still UnknownKey
- [x] repeat unrevoke → NotRevoked
- [x] CLI Unrevoke arm present
- [x] crypto tests + workspace PASS
- [x] clippy `-D warnings` PASS
- [x] originals unchanged
- [x] docs table remove/revoke/unrevoke
