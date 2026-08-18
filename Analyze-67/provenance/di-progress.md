# Deep-interview progress — Analyze-67

## Settled
- **Mechanism (Q1=A):** AIRA-native signed UDP packet (identity+addr+Ed25519); store if trusted+not revoked; not Ethereum discv5/ENR; no FIND_NODE.
- **Listen (Q2=A1):** Separate `peer discv listen` + `peer discv announce --to`; loopback default; TCP `peer listen` unchanged.

## Open
- Storage: same `peers/dht.json` vs separate file
- Bind policy details / apply-book from UDP
- Non-goals / decision boundaries
- Wire format (envelope vs compact signed blob)
