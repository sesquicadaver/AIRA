# Analyze-204 — CRP route events (QUEUE #169)

## Done
- EventType: `RouteSelected`, `RouteRejected`, `RouteFailed` (+ schema enum)
- `LocalCrpAdapter::route`/`bind` optionally emit via `EventSink`
- Test: `crp_route_events_selected_rejected_failure`
- QUEUE `#169` DONE → `#170` OPEN

## Out
B2-006 C3 case (`#170`); CRP status PARTIAL (`#171`).
