# Desktop P6 Advanced discovery (QUEUE #105)

Operator shortcuts for STUN reflexive query, UDP discv announce, and iterative FIND. **Dev / Advanced panel only** — not a dial or trust source.

## Fail-closed

- No public STUN default; `stun_server` must be explicit `host:port`.
- `discv announce` requires explicit `addr` (advertised listen address).
- FIND requires non-empty `key_ref` (`aira:identity:…`).
- Discovery results do **not** auto-trust or update AddressBook.

## Runtime API

```rust
run_stun_query(&paths, stun_server) -> DiscoveryStunOutcome
run_discv_announce(&paths, to, advertised_addr) -> String
run_discv_find(&paths, key_ref, to_opt, k) -> DiscvFindReport
```

Bootstrap runs before each call (identity + dirs). GUI panel is `#105` (`Discovery (P6 Dev)` in [`desktop-gui.md`](desktop-gui.md)).

## CLI parity

Underlying primitives live in `aira-peer` / `aira` peer CLI (`docs/peer-link.md`). Desktop wraps the same signed UDP paths without adding protocol fields.

## Related

- UX profile P6: [`desktop-ux.md`](desktop-ux.md)
- Plan: [`phase-e-plan.md`](phase-e-plan.md) §4d P6
- GUI: [`desktop-gui.md`](desktop-gui.md)
