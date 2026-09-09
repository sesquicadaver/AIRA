# Analyze-347 — Dial AddressBook candidate honesty

**QUEUE:** `#311` · **RFC-D:** [AIRA-RFC-0197](../../specs/rfc/AIRA-RFC-0197-addressbook-candidate-honesty.md)

## Outcome

Opt-in dial trial-upserts AddressBook for the attempt; on dial failure the pre-attempt book is restored. Known address A is preserved after failed probe B; a first-time failed dial leaves no orphan entry.
