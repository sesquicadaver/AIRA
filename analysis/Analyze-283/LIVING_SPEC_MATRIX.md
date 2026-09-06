# Analyze-283 — Live EVM (QUEUE #248)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Live JSON-RPC dial | `json_rpc_http`, `EvmRendezvousProvider` JsonRpc | `live_json_rpc_roundtrip_via_reference_server` | **DONE** |
| `use_local_double=false` not stub | `amoy_live` / `anvil_live` validate | `live_amoy_config_validates_without_stub_error` | **DONE** |
| Reference RPC surface | `ReferenceEvmRendezvousRpc` | same roundtrip | **DONE** |
| RFC-D | `AIRA-RFC-0140-…` | `phase_n_fix_*` | **DONE** |
| Two-process ab ovo | — | — | **OUT** (`#249`) |
