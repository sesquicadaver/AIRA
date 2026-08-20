# ULTRAQA — Analyze-93

**Verdict:** PASS (local)  
**Date:** 2026-08-20

## Goal
Local scan/list produces immutable inventory without network/download; scope escape denied; C1 untouched.

## Scenario matrix

| ID | Model | Scenario | Expected | Actual | Status |
|----|-------|----------|----------|--------|--------|
| U1 | CSU | manifest sandbox | scoped + network none | unit test | PASS |
| U2 | scan | gguf under models/ | installed≥1 artifact | CLI smoke | PASS |
| U3 | list | after scan | same artifact_id | CLI smoke | PASS |
| U4 | scope | `--dir /tmp` | fail exit 1 | CLI | PASS |
| U5 | out | downloadable list | empty array | unit test | PASS |
| U6 | C1 | conformance | failed=0 | aira-cli | PASS |
| U7 | firewall | dep graph | clean | dep_firewall.py | PASS |
