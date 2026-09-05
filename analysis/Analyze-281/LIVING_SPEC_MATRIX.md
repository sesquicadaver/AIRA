# Analyze-281 — NAT/relay (QUEUE #246)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Relay-only dial plan | `plan_inbound_blocked_relay_path` | `inbound_blocked_plan_is_relay_only` | **DONE** |
| Dual inbound blocked → courier | `configure_inbound_blocked_via_relay` + hub | `both_inbound_blocked_relay_courier_noise_succeeds` | **DONE** |
| RFC-D | `AIRA-RFC-0138-…` | `phase_n_rfc_0138_*` | **DONE** |
| RFC-0123 close | — | — | **OUT** (`#247`) |
