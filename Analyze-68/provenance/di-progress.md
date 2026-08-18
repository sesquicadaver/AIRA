# Deep-interview progress — Analyze-68

## Settled
- **Mechanism (Q1=A):** AIRA-native signed UDP FIND/NODES; iterative XOR-closest; merge local dht.json; not Ethereum.
- **UDP target (Q2=A1):** FIND sent to `addr` from dht.json as UDP host:port (same advertised addr).
- **Listen (Q3=B1):** existing `peer discv listen` multiplexes announce + FIND → NODES.

## Open
- apply-book from FIND results
- k / α / hop cap
- replay/freshness (A-67 advisory)
- Out / decision boundaries
