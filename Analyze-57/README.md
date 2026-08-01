# Analyze-57 — DHT → address_book (`--apply-book`)

**QUEUE:** #22  
**Status:** CLOSED (pending commit hash)  
**Decision:** both paths — find exact + inbound announce

## What shipped

- `AddressBook::upsert_addr_preserve_via`
- `promote_dht_to_address_book` / `apply_book_exact_from_dht_find` / `apply_dht_announce_maybe_book`
- CLI: `peer dht find --apply-book`; `peer listen --dht --apply-book`
- Docs + tests; UltraQA PASS; APPROVE/CLEAR

## Out

discv5; auto without flag; dial from DHT; XOR-closest auto-promote; #23+.
