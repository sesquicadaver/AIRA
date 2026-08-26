# Desktop federation join (QUEUE #103)

**Phase G:** federation join hardening (`#140`); C3 scaffold (`#141`). **Out:** remote federation runtime; settlement/CRP.

Local operator ceremony: signed federation descriptor JSON → TrustStore pin + `federation/membership.json`. Same semantics as `aira federation join --descriptor`.

## Runtime API

```rust
join_federation_descriptor_file(&paths, descriptor_path) -> JoinOutcome
read_federation_membership(&paths) -> Option<FederationMembership>
leave_federation_local(&paths) -> LeaveOutcome
```

Join verifies descriptor signature, pins TrustStore, writes membership, and appends `federation_join` to `identity/trust-audit.jsonl`. Rejoin with the same descriptor is idempotent (no duplicate audit). After `federation leave`, re-join writes fresh membership and a new audit entry.

## CLI parity

```bash
aira federation join --descriptor ./fed.json
aira federation leave
```

## Related

- Protocol: [`peer-link.md`](peer-link.md) (federation join pin A-70)
- Plan: [`phase-e-plan.md`](phase-e-plan.md) §4d P5
