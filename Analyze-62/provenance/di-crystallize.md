# DI crystallize — Analyze-62 / QUEUE #27

**Chosen:** **A**
- Path: `identity/tenants/<csu-safe-id>/ed25519` + `meta.json` (publisher_id, pubkey)
- Mode `0600` on secret
- API: save / load / load_all
- CLI for register+save / load
- Auto-load on `LocalSession::open` / node identity register path
- Keep in-memory isolation (`signature_for_tenant`)
- Out: tenant rotate ceremony (#28)
