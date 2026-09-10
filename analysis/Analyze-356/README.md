# Analyze-356 — AddressBook selective rollback

**QUEUE:** `#320` · **RFC-D:** [AIRA-RFC-0205](../../specs/rfc/AIRA-RFC-0205-addressbook-selective-rollback.md)

## Outcome

Failed dial rolls back only its candidate peer endpoint; parallel AddressBook upserts remain.
