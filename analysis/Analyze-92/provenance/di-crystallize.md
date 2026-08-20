# DI crystallize — Analyze-92 / QUEUE #57

## In
1. Execute only QUEUE `#57` (ModelAcquisitionPolicy payload schema).
2. RFC-S: CustomArtifact payload, not Core entity, not canonical enum.
3. Schema `$id` `aira:schema:model:acquisition-policy:0.1` with required `auto_download` (default false posture).
4. `schema validate --fixtures` and `aira-schema` tests green.

## Out
`#58`–`#60` implementation; downloader; allowlist runtime; Book 0 pipeline; `aira-core`.

## Ambiguity gate
Low: EVO-3 §2 policy example + Phase D D0.5 + prior model payload pattern are sufficient. No clarifying round required.
