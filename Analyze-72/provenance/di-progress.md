# Deep-interview progress — Analyze-72

## Facts (not rounds)
- `identity csu-tenant register|rotate --secret-hex` — 64-hex argv (demo-only; process list / history).
- A-62/A-63 deferred `--secret-hex-file` / stdin. QUEUE #37 Done when: secret з file або stdin, не argv.
- Out: HTTP authz. Prune уже #36.

## Settled
- **Input (Q1=A):** `--secret-hex-file PATH`; `PATH=-` = stdin. Взаємовиключно з `--secret-hex`. Порожній / не 32 байти — fail-closed. `--secret-hex` лишається як demo.

## Settled (cont.)
- **Encoding (Q2=A1):** файл/stdin — той самий 64-hex текст, що `--secret-hex` (не raw 32 байти).

## Settled (cont.)
- **Normalize (Q3=B1):** trim лише кінців (пробіли/один newline). Далі рівно 64 `[0-9a-fA-F]`. Без `0x`, без внутрішніх пробілів.

## Open
- Q4: non-goals / які команди.
