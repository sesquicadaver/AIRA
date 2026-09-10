# Analyze-352 — CLI identity create fail-closed

**QUEUE:** `#316` · **RFC-D:** [AIRA-RFC-0201](../../specs/rfc/AIRA-RFC-0201-cli-identity-create-fail-closed.md)

## Outcome

CLI and Desktop share `create_or_ensure_node_identity`. Exclusive create refuses overwrite; ensure is a no-op on a complete pair. Signature is validated before write.
