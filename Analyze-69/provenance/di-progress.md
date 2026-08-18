# Deep-interview progress — Analyze-69

## Settled
- **Opt-in (Q1=A):** `--allow-public-bind` required for non-loopback `--listen` and `--health-listen`. Without flag → fail-closed. Loopback unchanged.

## Settled (cont.)
- **Extra gates (Q2=A1):** flag is the only extra gate; TLS/Bearer stay independent; public+plain HTTP → warning not fail.

## Settled (cont.)
- **Out (Q3=B1):** federation; peer listen; require TLS/Bearer; public-by-default; port changes.

## Settled (cont.)
- **Boundaries (Q4=F1):** агент сам — ім’я `--allow-public-bind` (без env default); helper `assert_bind_allowed`; IPv6 `::1` = loopback; `0.0.0.0`/`::` потребують flag.

## Open
- (none — crystallize)
