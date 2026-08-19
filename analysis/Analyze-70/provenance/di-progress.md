# Deep-interview progress — Analyze-70

## Facts (not rounds)
- CLI: немає `federation`; bootstrap відкладає `aira federation join`.
- Trust уже є (`identity trust add`, peer trust-delta). Admission = local `trust.json`.
- `IdentityType::Federation` уже в `aira-protocol`.
- Book II §14: descriptor + join request/response. C3 ще має CRP — **Out цього рядка**.
- QUEUE Done when: мінімальний join+trust; Living Spec. Out: settlement/CRP.

## Settled
- **Join primitive (Q1=A):** локальний opt-in — підписаний federation descriptor → `federation/membership.json` + upsert `identity_ref` у TrustStore. CLI `aira federation join`. Без нового peer-повідомлення, без CRP/export.

## Settled (cont.)
- **Trust gate (Q2=A1):** самопідписаний descriptor — операторський ceremony. Verify Ed25519 проти pubkey в descriptor (`identity_ref`); потім upsert + membership. Revoked id → fail-closed.

## Settled (cont.)
- **Out (Q3=B1):** Join Request/Response (навіть файли); новий peer message; CRP / capability routing / import-export; settlement; Federation CSU; зміна hello/Noise/trust-delta; HTTP federation API; `federation leave` / exit / federation-revoke.

## Settled (cont.)
- **Re-join (Q4=C1):** один membership. Той самий `federation_id`+key → idempotent. Інший `federation_id` → fail-closed (без overwrite). Leave — не в цьому рядку.

## Settled (cont.)
- **Boundaries (Q5=F1):** агент сам — CLI `aira federation join --descriptor`; path `federation/membership.json`; domain `aira:federation:descriptor:v1`; descriptor subset; same-id different key fail-closed; логіка в `aira-protocol` (`Keyring::with_verifying_hex` у `aira-object`).

## Open
- (none — crystallize)
