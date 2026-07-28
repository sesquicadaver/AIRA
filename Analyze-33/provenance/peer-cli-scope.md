# Provenance — Peer CLI

## Decision
Wrap Analyze-32 `aira-peer` with `aira peer add|list|listen|dial|send`; fail-closed address-book add requires trust.

## Why
Operator path for decentralized links without Noise/X25519 in this cycle.

## Rejected
Noise XX, trust-delta wire sync, daemon multi-accept.

## Upstream tip
`398e15f`
