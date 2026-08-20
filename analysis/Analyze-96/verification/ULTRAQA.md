# UltraQA — Analyze-96 / QUEUE #61

## Scope
Hostile checks on policy ALLOW vs DENY; prove no transfer on ALLOW.

| ID | Scenario | Expect | Result |
|----|----------|--------|--------|
| U1 | no policy download | DENY + exit 2 + denied event | PASS (CLI smoke) |
| U2 | auto_download=false | DENY | PASS (unit) |
| U3 | auto_download=true | ALLOW + exit 0 + allowed event | PASS (unit + CLI) |
| U4 | ALLOW creates weights/quarantine | must not | PASS |
| U5 | dep_firewall CSU↛CSU | clean | PASS |

## Verdict
**PASS** — ready for PR.
