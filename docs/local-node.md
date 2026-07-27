# Local node guide

## Layout after `aira init`

```text
.aira/
  config.json
  identity/          # aira identity create + trust.json
  db/aira.sqlite
  artifacts/         # CAS + index.json
  csu/registry.json
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
cargo run -p aira-cli -- --root "$ROOT" identity rotate   # same id, new secret
# optional: identity rotate --backup  → identity/local.ed25519.prev
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

## Local HTTP API (Roadmap M11)

Default listen is loopback only (`127.0.0.1:8787`):

```bash
cargo run -p aira-node -- --root "$ROOT" --init --http --listen 127.0.0.1:8787
```

| Method | Path | Body / notes |
|--------|------|----------------|
| GET | `/health` | liveness |
| POST | `/v1/problems` | `{"text":"Calculate 2 + 2"}` |
| GET | `/v1/problems/:id` | problem status JSON |
| GET | `/v1/results/:id` | result payload |
| GET | `/v1/artifacts/:id` | descriptor + payload |
| GET | `/v1/events?limit=50` | event tail |
| GET | `/v1/capabilities` | local discovery seed |
| GET | `/v1/csu` | registry list |
| POST | `/v1/csu/register` | `{"manifest":{...},"activate":true}` |
| POST | `/v1/conformance/run` | `{"profile":"C0"}` |

Example:

```bash
curl -sS -X POST http://127.0.0.1:8787/v1/problems \
  -H 'content-type: application/json' \
  -d '{"text":"Calculate 2 + 2"}'
```

Non-goals for M11: auth hardening, TLS, multi-tenant, public bind by default, federation.

Peer-to-peer authenticated links (Analyze-32) are documented in [peer-link.md](peer-link.md); they are separate from this loopback HTTP API.

## Notes

- Config is JSON MVP (`config.json`); YAML parity is deferred.
- Multi-submit uses `run_nonce` so artifact/event ids do not collide.
- No network dependency is required for local MVP.
- AiraRefs in path segments may include `:` (single URL path segment).
