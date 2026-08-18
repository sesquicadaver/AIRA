# DI crystallize — Analyze-72 / QUEUE #37

## In scope (A + A1 + B1 + C1 + F1)
1. `--secret-hex-file PATH` on `identity csu-tenant register` and `rotate`.
2. `PATH=-` reads stdin; TTY stdin → fail-closed (no hang).
3. Mutually exclusive with `--secret-hex` (clap `conflicts_with`). `--secret-hex` stays (demo).
4. File/stdin body: trim ends only → exactly 64 `[0-9a-fA-F]` → 32-byte seed. No `0x`, no inner spaces, not raw bytes.
5. Empty / wrong length / oversize (>4KiB) / invalid hex → fail-closed. Never print the secret.
6. One shared CLI helper used by register and rotate.

## Out
`identity create` / `identity rotate` secret-file; env; remove `--secret-hex`; HTTP; tenant layout change; echo/print secret.

## Decision boundaries (agent-owned)
Helper in `aira-cli` (not `aira-object` layout). Read cap 4KiB. Docs: `docs/crypto.md`. Tests: parse + file + TTY-stdin fail; CLI xor flags.
