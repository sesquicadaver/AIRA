# AIRA-RFC-0011 — Local quarantine fetch after ALLOW (RFC-D/E)

## 1. Summary

After policy ALLOW (`#61`), `fetch_to_quarantine` copies a **local** `--source` file into `<root>/models/quarantine/<slot>/…`, publishes a quarantine receipt `CustomArtifact`, emits `op:quarantine-fetched:download:…`, and updates `models/quarantine.latest.json`. No hash/signature verify and no activation.

## 2. Problem Statement

ALLOW alone does not stage weights. D4.2 must prove a scoped local staging path before verify (`#63`) and activate (`#64`).

## 3. Scope

- `fetch_to_quarantine(root, model_ref, source)`
- Reject `http(s)://` / `ftp://` sources
- CLI: `aira models download --model-ref … --source <path>`
- Manifest `filesystem=scoped`, `network=none`

## 4. Non-Goals

```text
remote HTTP download
content_hash / signature verify (#63)
activate / inventory promote (#64)
sharing / rating
C1 / aira-core
```

## 5. Semantics

| Condition | Result | Exit |
|-----------|--------|------|
| DENY gate | no copy | 2 |
| ALLOW + local file | quarantine copy + Event | 0 |
| URL source | error `RemoteSource` | non-zero |

Receipt fields include `verified=false`, `activated=false`.

## 6. Rollback

Remove `fetch_to_quarantine` and CLI `--source`; restore gate-only download.

## 7. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6a D4.2; QUEUE `#62`.
