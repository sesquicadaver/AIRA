# DI crystallize — Analyze-60 / QUEUE #25

**Chosen:** **A** — systemd only; two units (`aira-node.service` HTTP loopback + `aira-peer-listen.service`) + short runbook in docs.

**Rationale:** systemd is the reliability-first supervisor on Linux; QUEUE names both long-running processes; Out = no runtime code.

**Non-goals:** supervisord tree; runtime changes; public bind (#34); cargo install automation.
