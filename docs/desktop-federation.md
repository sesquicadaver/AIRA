# Desktop federation join (QUEUE #103)

**Phase G:** `federation leave` CLI (`#134`); join hardening (`#135`). **Out:** remote federation runtime; settlement/CRP.

Local operator ceremony: signed federation descriptor JSON → TrustStore pin + `federation/membership.json`. Same semantics as `aira federation join --descriptor`.

## Runtime API

```rust
join_federation_descriptor_file(&paths, descriptor_path) -> JoinOutcome
read_federation_membership(&paths) -> Option<FederationMembership>
```

Bootstrap runs before join (identity + dirs). GUI wizard is `#104`.

## CLI parity

```bash
aira federation join --descriptor ./fed.json
```

## Related

- Protocol: [`peer-link.md`](peer-link.md) (federation join pin A-70)
- Plan: [`phase-e-plan.md`](phase-e-plan.md) §4d P5
