# Living Spec Matrix — Analyze-60

| ТЗ / QUEUE | Артефакт | Верифікація |
|------------|----------|-------------|
| #25 systemd unit example(s) | `deploy/systemd/aira-node.service`, `deploy/systemd/aira-peer-listen.service` | `systemd-analyze verify` (stub ExecStart); Type=simple; loopback binds |
| Short runbook | `docs/runbook-systemd.md` | links from README, local-node, peer-link |
| No runtime | crates unchanged | git diff crates/ empty of logic |
