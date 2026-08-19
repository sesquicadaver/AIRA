# Analyze-80 — core dependency firewall (QUEUE #45)

## Status
OPEN (implementation on branch `analyze-80-core-dependency-firewall`).

## Done when
CI fails on forbidden workspace edges: `aira-core` → `aira-node` / `aira-peer` / concrete CSU; concrete CSU → concrete CSU; directed import cycles. Current tree is green.

## Out
Mechanical file splits (#46+); new crates.
