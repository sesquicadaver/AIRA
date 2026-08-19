# Living Spec Matrix — Analyze-64

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Principal enum | `tenant_auth::Principal` | `unscoped_without_map`, `admin_when_not_in_map` |
| Map loader | `load_tenant_auth_map` | `duplicate_*`, `empty_*`, `map_mode_*` |
| Resolver | `resolve_principal` | `map_token_wins_over_admin_same_secret` |
| authorize register | `authorize_csu_register` | `authorize_and_filter`, `tenant_register_ok_and_cross_forbidden` |
| filter list | `filter_csu_list` | `tenant_list_filtered_admin_sees_all` |
| Boot helper | `validate_http_auth_boot` | `map_without_http_token_*`, `explicit_*_missing_*` |
| Legacy | HTTP no map | `legacy_no_map_unscoped_ok` |
