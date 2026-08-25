# Analyze-165 — Split desktop app (QUEUE #130)

## Status
OPEN — mechanical split `aira-desktop/src/app.rs` → `app/{mod,labels,profile,federation,discovery,invite,ui}.rs`.

## Done when
`cargo test -p aira-desktop`; clippy `-D warnings` on `aira-desktop`.

## Out
GUI redesign; behavior change.
