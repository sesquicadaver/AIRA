# Provenance — peer listen daemon scope

**Cycle:** Analyze-34  
**Parent:** Analyze-33 one-shot listen; QUEUE #1

## Decision
CLI persistent accept loop + optional `--recv`; remove idle timeout from `accept`'s TCP wait.

## Out of scope
Noise XX; trust-delta; NAT; gossip; DHT; supervisor/systemd unit.
