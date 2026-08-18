# Analyze-72 — Tenant `--secret-hex-file` / stdin (QUEUE #37)

## Status
CLOSED (implementation; QUEUE hash in close commit).

## Shipped
- `identity csu-tenant register|rotate --secret-hex-file PATH` (`-` = stdin, TTY fail-closed)
- `--secret-hex` remains demo; mutually exclusive

## Out
node identity secret-file; env; remove argv flag; HTTP
