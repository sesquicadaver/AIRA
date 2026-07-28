# TODO / FIXME — Analyze-31

## Blocking (code-review Cycle 3)
- [x] After successful trust upsert, backup `rename(tmp→.prev)` failure must not `restore_previous()` + destroy staging — **fixed in A-31** (see `verification/CODE_REVIEW.md`).

## Deferred → канон у [`QUEUE.md`](../../QUEUE.md)
- [ ] Dual-key grace for same node `key_ref` (multi-pubkey Keyring) → QUEUE #4
- [ ] CRL / ceremony audit log (rotate, revoke, unrevoke) → QUEUE #7
- [ ] Auto-notify peers of new node pubkey → QUEUE #5
- [ ] Timestamped backup rotation history (not single `.prev` slot) → QUEUE #8
