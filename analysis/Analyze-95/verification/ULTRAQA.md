# ULTRAQA — Analyze-95

**Verdict:** PASS (local)  
**Date:** 2026-08-20

## Scenario matrix

| ID | Scenario | Expected | Status |
|----|----------|----------|--------|
| U1 | no policy download | DENY + event | PASS |
| U2 | auto_download=false | DENY | PASS |
| U3 | auto_download=true | DENY, no weights | PASS |
| U4 | CLI download | exit 2 | PASS |
| U5 | dep_firewall | clean | PASS |
| U6 | C1 | green | PASS |
