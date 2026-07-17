# TODO / FIXME — Analyze-31

## Blocking (code-review Cycle 3)
- [ ] After successful trust upsert, backup `rename(tmp→.prev)` failure must not `restore_previous()` + destroy staging (trust≠secret + lost new key). Keep rotate committed; preserve `*.tmp`; add test with `.prev` as directory.

## Deferred
- [ ] Dual-key grace for same node `key_ref` (multi-pubkey Keyring)
- [ ] CRL / ceremony audit log (rotate, revoke, unrevoke)
- [ ] Auto-notify peers of new node pubkey
- [ ] Timestamped backup rotation history (not single `.prev` slot)
