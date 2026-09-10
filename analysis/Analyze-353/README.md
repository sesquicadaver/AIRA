# Analyze-353 — Policy audit uniqueness

**QUEUE:** `#317` · **RFC-D:** [AIRA-RFC-0202](../../specs/rfc/AIRA-RFC-0202-policy-audit-uniqueness.md)

## Outcome

Policy (and companion CSU life/fail) event ids embed a run nonce. Persist no longer silently skips same-id events with a different canonical hash.
