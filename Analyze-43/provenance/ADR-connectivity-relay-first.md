# ADR — Connectivity roadmap (Analyze-43)

## Status
Accepted for follow-up slices; not implemented here.

## Context
Reliable ETH/TRON-like connectivity needs discovery + transport + NAT strategy + gossip. AIRA has Noise + address book + trust-delta.

## Decision
1. **Now (A-43):** gossip fanout + local discovery journal
2. **Next:** **relay-first** connectivity (trusted relay peer in address book) before STUN/ICE
3. **Later:** DHT / discv-style discovery once relay + gossip are stable

## Consequences
- Nodes behind NAT get a path without hole punching first
- DHT is deferred until there is something useful to announce/find
