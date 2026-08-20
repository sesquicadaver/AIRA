# UltraQA — Analyze-98 / QUEUE #63

| ID | Scenario | Expect | Result |
|----|----------|--------|--------|
| U1 | signed match | verified staging, exit 0 | PASS |
| U2 | wrong hash | reject, quarantine kept | PASS |
| U3 | TESTSIG | unsigned reject | PASS |
| U4 | activated flag | always false | PASS |
| U5 | dep_firewall | clean | PASS |

## Verdict
**PASS**
