# Provenance — Node secret durable backup

## Decision
Opt-in `--backup` writes `local.ed25519.prev` + meta sidecar before rotate overwrite; fail closed on backup I/O errors.

## Why
Operator recovery after Analyze-30 cutover without always leaving secrets on disk.

## Rejected
Always-on backup; dual-key same-ref; peer notify.

## Upstream tip
`935fd3d` (Analyze-30)
