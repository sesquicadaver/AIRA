# Analyze-58 — Durable relay hub registry

**QUEUE:** #23  
**Status:** CLOSED (`f1e311f`)  
**Decision:** keep offline history (**A**) + optional TTL (31d recommended)

## Shipped
- `peers/relay_hub.json` (`RelayHubRegistry`) with process-wide RMW lock
- Durable mark online **before** live register; offline on disconnect
- Optional `--relay-ttl-days` (offline-only prune); pre-bind writable probe
- Live routes remain RAM-only

## Out
STUN; session crypto resurrect; undelivered queue; #24+
