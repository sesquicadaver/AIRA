# AIRA-RFC-0092 — Envelope freshness and replay window

## 1. Summary

Phase I `#194`: peer `recv_envelope` admits a signed envelope only if `created_at` is within a policy skew of now, `expires_at` (when set) is still in the future, and `message_id` has not been seen inside the replay TTL. Gossip `gossip_seen.json` is unchanged (forward-once, not receive admission).

## 5. Non-Goals

Run-nonce concurrency (`#195`); instance-scoped crypto (`#196`); requiring `expires_at` on every historical fixture; rewriting in-process C2 adapters.

## 10. Contract

```text
recv (after signature) → admit_envelope(now, policy)
  expires_at set and now >= expires_at → Expired
  |now − created_at| > 300s → ClockSkew
  message_id seen within 600s → Duplicate / Replay
persist peers/envelope_replay.json (cap 4096)
senders may omit expires_at; skew still applies
```

## 15. Tests

```text
cargo test -p aira-protocol --lib admit
cargo test -p aira-peer --lib recv_envelope_rejects_expired
cargo test -p aira-peer --lib recv_envelope_rejects_replayed_message_id
cargo test -p aira-peer --lib admit_received
```
