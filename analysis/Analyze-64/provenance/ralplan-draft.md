# Ralplan — Analyze-64 / QUEUE #29 (rev2)

## Principles
1. Authn unchanged; authz on CSU HTTP routes only.
2. Mismatch → **403**; missing/wrong Bearer → **401**.
3. Opt-in map; no map → `Principal::Unscoped`.
4. Fail closed on bad map / inconsistent boot.

## Decisions (B1–B7)
| # | Decision |
|---|----------|
| B1 | Map entry wins if token also equals `--http-token`. Duplicate tokens → load Err. Empty token/publisher → load Err. |
| B2 | Full linear scan + `constant_time_eq` (no early exit). |
| B3 | Write `0600`; load rejects looser mode (unix). |
| B4 | Exact `AiraRef` equality on `publisher_identity`. |
| B5 | `Principal::{ Admin, Tenant { publisher_id }, Unscoped }`. |
| B6 | Map present + no `--http-token` → boot fail. |
| B7 | mTLS CN→Principal **OUT**; **no seam this row**. Follow-up must add CN→`Request::extensions` separately. |

## AppState contract
- Load map once at process start (immutable for lifetime). No SIGHUP reload this row.
- `http_token: Option<Arc<str>>` + `tenant_auth: Option<TenantAuthMap>` on `AppState`.
- After Bearer match: `resolve_principal(token) -> Principal` (Admin / Tenant / if no map Unscoped only when token gate off).

## Scope
1. Module `crates/aira-node/src/tenant_auth.rs` (load/resolve/authorize/filter).
2. Default path `$ROOT/identity/http-tenant-auth.json` if exists; `--http-tenant-auth PATH` requires file.
3. Wire `POST /v1/csu/register` + `GET /v1/csu`.
4. Boot helper `validate_http_auth_boot(token, map_path) -> Result`.
5. Docs + QUEUE + Living Spec.

## Living Spec rows
| ТЗ | Модуль | Тест |
|----|--------|------|
| Principal enum | `tenant_auth::Principal` | resolve unit |
| Map loader | `load_tenant_auth_map` | dup/empty/mode |
| Resolver | `resolve_principal` | map-wins / admin / unscoped |
| authorize register | `authorize_csu_register` | ok / 403 |
| filter list | `filter_csu_list` | filtered / admin all |
| Boot helper | `validate_http_auth_boot` | map w/o token; explicit missing |

## Tests (must)
- legacy_no_map_unscoped_ok
- admin_token_full_access
- map_token_wins_over_admin_same_secret
- tenant_register_ok / cross_tenant_403
- tenant_list_filtered / admin_list_all
- duplicate_token_in_map_rejects_load
- empty_token_or_publisher_rejects_load
- map_mode_rejects_world_readable (unix)
- map_without_http_token_boot_error
- explicit_tenant_auth_path_missing_boot_error
- bearer_still_401_without_token

## Out
mTLS CN principal + seam; federation; problems/events tenancy; #36 prune; SIGHUP reload.
