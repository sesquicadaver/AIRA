# UltraQA — Analyze-60

Docs/deploy-only slice (no runtime). Adversarial static checks:

| ID | Scenario | Result | Evidence |
|----|----------|--------|----------|
| S1 | Unit structure with stub ExecStart=/bin/true | PASS | systemd-analyze verify exit 0 |
| S2 | Repo units placeholder binaries | PASS | only expected “not executable” |
| S3 | No public bind in ExecStart | PASS | both use 127.0.0.1 |
| S4 | Type=simple + Restart=on-failure present | PASS | both units |
| S5 | Runbook PORT ownership + must/optional | PASS | docs/runbook-systemd.md |
| S6 | No crate runtime changes | PASS | crates/ untouched this slice |
| S7 | Doc links README/local-node/peer-link | PASS | spot-check |

**Verdict:** CLEAR
