# Verification Checklist — Analyze-28

- [x] rotate without until → old not in keyring
- [x] rotate with future until → old+new verify at now
- [x] after until → only new
- [x] invalid --until / timestamp → error
- [x] crypto tests (9) + workspace PASS
- [x] clippy `-D warnings` PASS
- [x] originals unchanged
- [x] docs updated
