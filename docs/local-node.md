# Local node guide

## Layout after `aira init`

```text
.aira/
  config.json
  identity/          # aira identity create + trust.json + trust-audit.jsonl
  db/aira.sqlite
  artifacts/         # CAS + index.json
  csu/registry.json
  discovery/registry.json  # local capability discovery (Analyze-45)
  http/              # optional self-signed TLS PEM (`--tls-self-signed`)
  events/event-log.json
  problems/index.json
  conformance/reports/
```

## Setup

```bash
cargo build -p aira-cli -p aira-node
ROOT=/tmp/aira-node
cargo run -p aira-cli -- --root "$ROOT" init
cargo run -p aira-cli -- --root "$ROOT" identity create --name local
cargo run -p aira-cli -- --root "$ROOT" identity rotate   # same id, new Ed25519 + local.x25519
# optional: identity rotate --backup  → local.ed25519.prev + local.x25519.prev (+ archives)
cargo run -p aira-cli -- --root "$ROOT" identity trust list
cargo run -p aira-cli -- --root "$ROOT" status
```

## Problem → result (CLI)

```bash
cargo run -p aira-cli -- --root "$ROOT" problem submit --text "Calculate 2 + 2"
cargo run -p aira-cli -- --root "$ROOT" problem status <problem_ref>
cargo run -p aira-cli -- --root "$ROOT" result get <problem_ref>
cargo run -p aira-cli -- --root "$ROOT" event tail --limit 20
```

## aira-node (one-shot)

```bash
cargo run -p aira-node -- --root "$ROOT" --text "Calculate 2 + 2"
```

Loads config, lists CSU registry entries, runs one local OperationalPlane submit.

## Local HTTP API (Roadmap M11 + Analyze-45/48)

Default listen is loopback only (`127.0.0.1:8787`). Plain HTTP remains the default; TLS and Bearer auth are opt-in.

```bash
cargo run -p aira-node -- --root "$ROOT" --init --http --listen 127.0.0.1:8787
# HTTPS with generated self-signed PEM under $ROOT/http/
cargo run -p aira-node -- --root "$ROOT" --http --listen 127.0.0.1:8787 --tls-self-signed
# HTTPS with operator-provided PEM
cargo run -p aira-node -- --root "$ROOT" --http --listen 127.0.0.1:8787 \
  --tls-cert /path/cert.pem --tls-key /path/key.pem
# mTLS: require client cert signed by CA (Analyze-51). CN must be a full AiraRef
# present in TrustStore and not revoked (Analyze-55). ALL routes on --listen need client cert.
cargo run -p aira-node -- --root "$ROOT" --http --listen 127.0.0.1:8787 \
  --tls-self-signed --tls-client-ca /path/client-ca.pem
# Optional plain-HTTP liveness (Analyze-56): only GET /health, no client cert.
# Requires --tls-client-ca. Loopback only until QUEUE #34.
cargo run -p aira-node -- --root "$ROOT" --http --listen 127.0.0.1:8787 \
  --tls-self-signed --tls-client-ca /path/client-ca.pem \
  --health-listen 127.0.0.1:8788
# Optional Bearer auth for /v1/* (`/health` stays open at HTTP layer); also AIRA_HTTP_TOKEN
cargo run -p aira-node -- --root "$ROOT" --http --http-token "$TOKEN"
```

Client certificate **CN** = `aira:identity:…` that already exists in `$ROOT/identity/trust.json` (not on CRL). CA membership alone is not enough.

With `--health-listen`, probes use the separate bind; the API `--listen` surface stays mTLS-only.
| Method | Path | Body / notes |
|--------|------|----------------|
| GET | `/health` | liveness (no Bearer; under mTLS on `--listen` still needs client cert; use `--health-listen` for plain probe) |
| POST | `/v1/problems` | `{"text":"Calculate 2 + 2"}` |
| GET | `/v1/problems/:id` | problem status JSON |
| GET | `/v1/results/:id` | result payload |
| GET | `/v1/artifacts/:id` | descriptor + payload |
| GET | `/v1/events?limit=50` | event tail |
| GET | `/v1/capabilities` | local discovery (durable `discovery/registry.json`) |
| GET | `/v1/csu` | registry list |
| POST | `/v1/csu/register` | `{"manifest":{...},"activate":true}` |
| POST | `/v1/conformance/run` | `{"profile":"C0\|C1\|C2"}` |

Example:

```bash
curl -sS -X POST http://127.0.0.1:8787/v1/problems \
  -H 'content-type: application/json' \
  -d '{"text":"Calculate 2 + 2"}'
# with --tls-self-signed:
curl -skS https://127.0.0.1:8787/health
# with --http-token:
curl -sS http://127.0.0.1:8787/v1/capabilities -H "Authorization: Bearer $TOKEN"
# with --tls-client-ca (mTLS) on API listen:
curl -skS --cert client.pem --key client.key https://127.0.0.1:8787/health
# with --health-listen (plain probe, no client cert):
curl -sS http://127.0.0.1:8788/health
```

Non-goals (див. [`QUEUE.md`](../QUEUE.md)): multi-tenant HTTP authz (#29); public bind opt-in (#34); federation (#35); YAML `config.yaml` parity (#30).

Shipped на HTTP: mTLS require + CN→TrustStore (A-51/55); plain `--health-listen` (A-56 / #21).

Peer-to-peer authenticated links (Analyze-32…59) — [peer-link.md](peer-link.md); окремо від loopback HTTP API.

## Notes

- Config is JSON MVP (`config.json`); YAML parity — QUEUE #30.
- Multi-submit uses `run_nonce` so artifact/event ids do not collide.
- No network dependency is required for local MVP.
- AiraRefs in path segments may include `:` (single URL path segment).
- Long-running under systemd: [runbook-systemd.md](runbook-systemd.md) (`aira-node.service`).
