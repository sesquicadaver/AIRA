# UltraQA — Analyze-97 / QUEUE #62

| ID | Scenario | Expect | Result |
|----|----------|--------|--------|
| U1 | ALLOW + local `--source` | quarantine file + Event + exit 0 | PASS |
| U2 | no policy + `--source` | DENY, no quarantine dir | PASS |
| U3 | `https://…` source | RemoteSource error | PASS |
| U4 | gate without `--source` | policy-allowed only | PASS (compat) |
| U5 | dep_firewall | clean | PASS |

## Verdict
**PASS**
