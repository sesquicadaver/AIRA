# LIVING_SPEC_MATRIX — Analyze-57

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Preserve via on promote | `AddressBook::upsert_addr_preserve_via` | `upsert_addr_preserve_via_keeps_courier` |
| Promote API | `promote_dht_to_address_book` | `promote_without_prior_book_inserts`, `promote_rejects_bad_addr` |
| Announce + apply-book → dial | integration | `dht_announce_apply_book_then_dial` |
| Exact-only find promote | `apply_book_exact_from_dht_find` | `apply_book_exact_from_find_skips_closest_only` |
| CLI `--apply-book` requires `--dht` | aira-cli listen | UltraQA S1 |
| Docs / QUEUE #22 | peer-link, QUEUE | manual |
