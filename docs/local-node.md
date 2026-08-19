# Local node guide

## Layout after `aira init`

```text
.aira/
  config.json        # written by `aira init` (canonical)
  # config.yaml      # optional read-equivalent alternative (xor with config.json)
  identity/          # aira identity create + trust.json + trust-audit.jsonl
  db/aira.sqlite
  artifacts/         # CAS + index.json
  federation/membership.json  # optional local join pin (Analyze-70)
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

Loads config, lists CSU registry entries, runs one local **C1 reference** `OperationalPlane` submit ([operational-plane.md](operational-plane.md); not production event/scheduler/federation runtime).

## Local HTTP API (Roadmap M11 + Analyze-45/48)

Default listen is loopback only (`127.0.0.1:8787`). Non-loopback `--listen` / `--health-listen` (`0.0.0.0`, `::`, LAN, public) require `--allow-public-bind`; without it the process exits before bind. Plain HTTP remains the default; TLS and Bearer auth are independent opt-in (public + no TLS prints a warning).

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
# Requires --tls-client-ca. Non-loopback needs --allow-public-bind (Analyze-69).
cargo run -p aira-node -- --root "$ROOT" --http --listen 127.0.0.1:8787 \
  --tls-self-signed --tls-client-ca /path/client-ca.pem \
  --health-listen 127.0.0.1:8788
# Public bind is fail-closed without the flag:
# cargo run -p aira-node -- --root "$ROOT" --http --listen 0.0.0.0:8787
#   → error: pass --allow-public-bind
# cargo run -p aira-node -- --root "$ROOT" --http --listen 0.0.0.0:8787 --allow-public-bind
# Optional Bearer auth for /v1/* (`/health` stays open at HTTP layer); also AIRA_HTTP_TOKEN
cargo run -p aira-node -- --root "$ROOT" --http --http-token "$TOKEN"
# Multi-tenant CSU authz (Analyze-64): map Bearer → publisher_id
# File mode must be 0600. Requires --http-token (admin). Map tokens also accepted.
cat > "$ROOT/identity/http-tenant-auth.json" <<'EOF'
{"version":1,"entries":[{"token":"tenant-secret","publisher_id":"aira:identity:worker-pub"}]}
EOF
chmod 600 "$ROOT/identity/http-tenant-auth.json"
cargo run -p aira-node -- --root "$ROOT" --http --http-token "$ADMIN_TOKEN"
# or explicit path:
# cargo run -p aira-node -- --root "$ROOT" --http --http-token "$ADMIN_TOKEN" \
#   --http-tenant-auth "$ROOT/identity/http-tenant-auth.json"
```

`POST /v1/csu/register` with a tenant token → **403** if `manifest.publisher_identity` ≠ mapped publisher. `GET /v1/csu` returns only that publisher’s CSUs. Admin token (not in map) sees/registers all. mTLS CN→tenant principal is deferred (CN not yet in HTTP request extensions).

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
| GET | `/v1/csu` | registry list (tenant-filtered when map enabled) |
| POST | `/v1/csu/register` | `{"manifest":{...},"activate":true}` (tenant publisher must match) |
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

Non-goals (див. [`QUEUE.md`](../QUEUE.md)): mTLS CN principal seam; YAML **write**/convert CLI; SQLite audit rewrite.

Shipped на HTTP: mTLS require + CN→TrustStore (A-51/55); plain `--health-listen` (A-56 / #21); multi-tenant CSU authz Bearer map (A-64 / #29); public bind opt-in `--allow-public-bind` (A-69 / #34).

Peer-to-peer authenticated links (Analyze-32…59) — [peer-link.md](peer-link.md); окремо від HTTP API.

## Notes

- Config: `config.json` (from `aira init`) **or** `config.yaml` (same `NodeConfig` schema). Both present → fail-closed. Init never writes YAML.
- Multi-submit uses `run_nonce` so artifact/event ids do not collide.
- No network dependency is required for local MVP.
- Operational submit uses the C1 **reference/demo** plane, not a production event runtime ([operational-plane.md](operational-plane.md)).
- AiraRefs in path segments may include `:` (single URL path segment).
- Long-running under systemd: [runbook-systemd.md](runbook-systemd.md) (`aira-node.service`).
- Federation join (local pin): `aira federation join --descriptor` — [peer-link.md](peer-link.md).
