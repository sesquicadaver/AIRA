# ULTRAQA — Analyze-71

**Verdict:** PASS  
**Date:** 2026-08-18

## Goal
Tenant archive GC via `identity csu-tenant backups prune`; never latest/live; node prune does not touch tenant files.

## Scenario matrix
| ID | Scenario | Expected | Actual | Status |
|----|----------|----------|--------|--------|
| T1 | `cargo test -p aira-object --lib tenant::` | 23 pass | 23 pass | PASS |
| U1 | `identity csu-tenant backups --help` | prune subcommand | ok | PASS |
| U2 | backups without init | exit 1 | exit 1 | PASS |
| U3 | init + empty list | empty line + `tenant_backups` | ok | PASS |
| U4 | prune without flags | exit 1 | exit 1 | PASS |
| U5 | keep=1 deletes `9` keeps `10`+latest; dry-run no delete | as planned | ok | PASS |
| U6 | `identity backups prune --keep 0` leaves tenant archive | file remains | ok | PASS |
| U7 | keep=0 drops archive, latest `.prev` + `.tmp` + orphan meta remain | ok | ok | PASS |

Hostile: `.tmp` staging, orphan meta, lex-trap stamps `9`/`10`. Cleanup: temp root removed.
