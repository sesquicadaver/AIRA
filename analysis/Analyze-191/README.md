# Analyze-191 — durable event backend (QUEUE #156)

## Status
DONE — `aira_event::FileChainEventLog` (JSON file-chain).

## Done when
`open_or_create` / `append` / reopen roundtrip; tamper on disk rejected; unit tests green.

## Out
LocalSession / init_node wire (#157); SQLite events (optional later); scheduler.
