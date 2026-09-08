# Analyze-321 — Actionable Connection UX gap (Phase R plan)

**Статус:** problem analysis → [`docs/phase-r-plan.md`](../../docs/phase-r-plan.md) **складено, не активовано**  
**QUEUE:** немає OPEN (QUEUE Q closed); Phase R `#286`–`#294` лише в плані до wiring

## Problem (one paragraph)

Desktop після Phase O/P показує багато honesty-copy (strip, Connection, mesh banner, Saved/Applied, `help:`/`try:`), але job «під’єднатись до учасника» похований під Technical details і Help «розумій контракт». Explain ≫ instruct; cold-start виглядає як непотріб з поясненнями, хоча Settings є.

## Evidence

- `ui.rs`: Connection CTA Refresh; `ui_network_ops` under tech collapse
- `docs/help/uk/network.connect.md`: tech details + contract, not invite scenario
- Phase P invariants must stay; change presentation priority only

## Fix plan

See Phase R atoms `#286`–`#294` in `docs/phase-r-plan.md`. Do not start implementation until developer activates wiring `#286`.
