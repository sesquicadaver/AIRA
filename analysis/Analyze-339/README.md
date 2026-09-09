# Analyze-339 — Observe miss off UI-thread

**QUEUE:** `#303` · **RFC-D:** [AIRA-RFC-0190](../../specs/rfc/AIRA-RFC-0190-observe-miss-off-ui.md)

## Outcome

UI `observe()` on cache miss returns pending and schedules `observe_verify_now()` off-thread with `ContentHash::sha256_path`. Admit still hashes every time.
