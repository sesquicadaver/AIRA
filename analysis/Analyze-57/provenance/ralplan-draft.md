# Ralplan — Analyze-57 (consensus revision 1)

## Principles
1. Address book remains dial authority; DHT stays advisory
2. Opt-in only (`--apply-book`); no mutation without flag
3. Exact find + verified inbound announce only (no closest promote)
4. Preserve `via` on addr overwrite when peer already in book
5. CLI fail-closed: `listen --apply-book` **requires** `--dht` (same class as other side-effect flag deps)

## Chosen: A (revised)

| Surface | Behavior |
|---------|----------|
| `peer dht find --apply-book` | Mutate book **only** on exact DHT hit; upsert addr; preserve existing `via` |
| `peer listen --dht --apply-book` | After successful `apply_dht_announce`, same upsert; **bail** if `--apply-book` without `--dht` |
| Without flag | A-47 unchanged |

### Library API
Add `AddressBook::upsert_addr_preserve_via(id, addr)` (or equivalent): update/insert addr; if peer existed, keep `via`; if new, `via=None`. **Do not** use plain `upsert()` for promote (it clears `via`).

### Listen symmetry
Apply `--apply-book` in **both** recv paths (once + background), same as `--dht` apply today.

## Test-spec (required)
1. Exact find + `--apply-book` → book entry → dial ok  
2. Inbound announce + `--dht --apply-book` → book → dial ok  
3. Without flag → book unchanged after find/announce  
4. Closest-only (no exact) + `--apply-book` → **no** book mutation  
5. Existing peer with `via` → promote new addr → **via preserved**  
6. `listen --apply-book` without `--dht` → CLI error  

## Out
discv5; auto; dial from DHT; closest→book; #23+

## Prior lane notes
- Architect: APPROVE + WATCH (via + dual recv) — addressed above  
- Critic: REQUEST CHANGES — addressed; awaiting re-approve  
