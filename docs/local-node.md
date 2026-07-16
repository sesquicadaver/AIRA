# Local node guide

## Layout after `aira init`

```text
.aira/
  config.json
  identity/          # aira identity create
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
cargo run -p aira-cli -- --root "$ROOT" status
```

## Problem → result

```bash
cargo run -p aira-cli -- --root "$ROOT" problem submit --text "Calculate 2 + 2"
cargo run -p aira-cli -- --root "$ROOT" problem status <problem_ref>
cargo run -p aira-cli -- --root "$ROOT" result get <problem_ref>
cargo run -p aira-cli -- --root "$ROOT" event tail --limit 20
```

## aira-node

```bash
cargo run -p aira-node -- --root "$ROOT" --text "Calculate 2 + 2"
```

Loads config, lists CSU registry entries, runs one local OperationalPlane submit.

## Notes

- Config is JSON MVP (`config.json`); YAML parity is deferred.
- Multi-submit uses `run_nonce` so artifact/event ids do not collide.
- No network dependency is required for local MVP.
