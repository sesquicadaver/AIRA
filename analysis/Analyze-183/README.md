# Analyze-183 — SEC-2 canonical envelope signature (QUEUE #135)

## Status
OPEN — security audit blocker before relay/federation exposure.

## Done when
`ProtocolEnvelope` + `ProtocolResponse` canonical sign/verify over full descriptor; `signature.key_ref == issuer_identity`; C2 mutation tests per semantic field.

## Out
Wire protocol version negotiation; network deployment.
