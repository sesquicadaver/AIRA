# Living Spec Matrix — Analyze-315 / QUEUE `#280`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| dial≠accept local_addr один challenge | `export_from_session` + session bind | `nat_accept_local_addr_differs_from_challenge_still_proves_direct` |
| NAT path не fail-closed хибно | inbound no `endpoints_equivalent` gate | same + `endpoint_mismatch` (outbound still rejects) |
| Outbound strict | `issue_for_authenticated_session` | `endpoint_mismatch_rejects_issue_and_evidence` |
| RFC-0169 | `specs/rfc/AIRA-RFC-0169-…` | `phase_q_rfc_0169_present` |
