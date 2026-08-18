# Deep-interview progress — Analyze-71

## Facts (not rounds)
- Node: `identity backups prune --keep/--older-than-days/--dry-run` GC `local.ed25519.prev.<stamp>` + `local.x25519.prev.<stamp>`; never latest `.prev`.
- Tenant rotate `--backup` already archives `identity/tenants/<hex>/ed25519.prev.<stamp>`; prune API/CLI відсутні.
- `--secret-hex` на `csu-tenant register|rotate` — argv (demo-only). A-62/A-63 відклали `--secret-hex-file` / stdin.
- QUEUE Done when: prune CLI parity з node backups; тести. «optional stdin secret» у scope-колонці, не в Done when.
- Out цього рядка: HTTP authz.

## Settled
- **Scope (Q1=A):** лише tenant `ed25519.prev.<stamp>` prune. Stdin/`--secret-hex-file` — новий OPEN-рядок у кінець черги (не в #36).

## Settled (cont.)
- **CLI (Q2=A1):** `identity csu-tenant backups prune` з `--keep` / `--older-than-days` / `--dry-run`; обхід усіх `identity/tenants/<hex>/`; latest `ed25519.prev` не чіпати. `identity backups prune` без змін. Без `--csu-id`.

## Settled (cont.)
- **Keep/age (Q3=B1):** per-tenant, як node per-family. `--keep N` / `--older-than-days D` застосовуються окремо в кожному `tenants/<hex>/`. Latest `ed25519.prev` у кожному dir недоторканний.

## Settled (cont.)
- **Out/In (Q4=C1):** Out — stdin/`--secret-hex-file`; HTTP authz; зміна `identity backups prune`; auto-prune на rotate; `--csu-id`; зміна rotate/revoke/register; TrustStore/CRL; видалення latest `.prev` або live `ed25519`. In — `identity csu-tenant backups` (list) + `… prune`.

## Settled (cont.)
- **Boundaries (Q5=F1):** агент сам — політика як у node; логіка в `tenant.rs`; list колонки `csu_id`+stamp+path; QUEUE #37 = stdin. Питати лише TrustStore / rotate / різати latest `.prev`.

## Open
- (none — crystallize)
