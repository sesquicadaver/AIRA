# systemd runbook (Analyze-60 / QUEUE #25)

Example **systemd** units for long-running AIRA processes. **No runtime code** ships in this slice — copy, edit paths, enable.

Canonical files:

- [`deploy/systemd/aira-node.service`](../deploy/systemd/aira-node.service) — `aira-node --http` (loopback)
- [`deploy/systemd/aira-peer-listen.service`](../deploy/systemd/aira-peer-listen.service) — `aira peer listen --recv` (loopback)

Peer protocol details: [peer-link.md](peer-link.md). HTTP API: [local-node.md](local-node.md).

## Prerequisites

1. Build or install binaries (`aira-node`, `aira` from `aira-cli`).
2. Create a node root and identity (once per data dir):

```bash
ROOT=/var/lib/aira/node   # or /var/lib/aira/peer
sudo mkdir -p "$ROOT"
sudo chown aira:aira "$ROOT"   # after creating user `aira`
cargo run -p aira-cli -- --root "$ROOT" init
cargo run -p aira-cli -- --root "$ROOT" identity create --name local
```

3. Edit each unit: `User`/`Group`, `WorkingDirectory`, `--root`, `ExecStart` binary path.
4. **Peer PORT:** owned by the operator. Default example is `127.0.0.1:7900`. Change if occupied (`ss -ltn | grep 7900`). Put the same addr in peers’ `address_book.json`.

## Install & enable

```bash
sudo cp deploy/systemd/aira-node.service /etc/systemd/system/
sudo cp deploy/systemd/aira-peer-listen.service /etc/systemd/system/
# edit paths in /etc/systemd/system/aira-*.service
sudo systemctl daemon-reload
sudo systemctl enable --now aira-node.service
sudo systemctl enable --now aira-peer-listen.service
sudo systemctl status aira-node.service aira-peer-listen.service
```

Logs:

```bash
journalctl -u aira-node.service -f
journalctl -u aira-peer-listen.service -f
```

Verify unit syntax (when `systemd-analyze` is available):

```bash
systemd-analyze verify deploy/systemd/aira-node.service
systemd-analyze verify deploy/systemd/aira-peer-listen.service
```

Until binaries exist at the `ExecStart=` paths, verify reports “not executable” — expected for templates. After editing paths to real binaries (or verifying with a temporary `ExecStart=/bin/true` copy), the unit structure should pass.

## Must vs optional

| Class | Items |
|-------|--------|
| **Must** (in examples) | `Type=simple`; loopback `--listen` / `--bind`; `Restart=on-failure`; `NoNewPrivileges`; `PrivateTmp` |
| **Optional** | `ProtectSystem=strict` + `ReadWritePaths=…`; TLS/mTLS/`--health-listen`; Bearer `AIRA_HTTP_TOKEN`; peer `--relay` / `--dht` / `--apply-trust` |

Commented TLS/mTLS lines in the unit files are **examples only** — they require operator PEMs and are not a default security posture.

`After=network.target` only orders startup relative to that target; it does **not** guarantee the network or peers are ready.

## Out of scope

Runtime changes; supervisord configs; packaging/install scripts; SELinux policy. Public HTTP bind is opt-in (`aira-node --allow-public-bind`); this unit stays loopback.
