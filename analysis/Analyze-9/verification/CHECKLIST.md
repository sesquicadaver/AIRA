# Verification Checklist — Analyze-9

## Immutability
- [x] Manifesto etc / Meditation_About unchanged

## Epic 2
- [x] docs/canonical-terminology.md
- [x] schema directories + JSON files
- [x] aira-schema tests
- [x] `aira schema list|validate|--fixtures`

## Commands

```bash
cargo test -p aira-schema
cargo run -p aira-cli -- schema validate --fixtures fixtures
bash Analyze-9/verification/deny-originals.sh
bash Analyze-9/verification/deny-foreign-workspace.sh
```

## Result

```text
cargo test -p aira-schema → 6 passed
aira schema validate --fixtures → passed=22 failed=0
GPU object_type fixture → FAIL (expected)
deny-originals / deny-foreign → OK
Manifesto etc / Meditation_About → unchanged
```

**Verdict:** PASS
