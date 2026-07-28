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
# Optional Bearer auth for /v1/* (`/health` stays open); also AIRA_HTTP_TOKEN
cargo run -p aira-node -- --root "$ROOT" --http --http-token "$TOKEN"
```

| Method | Path | Body / notes |
|--------|------|----------------|
| GET | `/health` | liveness (no Bearer required) |
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
```

Non-goals (still deferred): mTLS, multi-tenant HTTP authz, public bind by default, federation.

Peer-to-peer authenticated links (Analyze-32+) are documented in [peer-link.md](peer-link.md); they are separate from this loopback HTTP API.

## Notes

- Config is JSON MVP (`config.json`); YAML parity is deferred.
- Multi-submit uses `run_nonce` so artifact/event ids do not collide.
- No network dependency is required for local MVP.
- AiraRefs in path segments may include `:` (single URL path segment).
