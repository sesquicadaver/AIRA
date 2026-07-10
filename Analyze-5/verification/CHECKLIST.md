# Verification Checklist — Analyze-5

## Immutability

- [x] Немає змін у `Manifesto etc/`
- [x] Немає змін у `Meditation_About/`

## Repo boundary

- [x] `deny-foreign-workspace.sh` PASS
- [x] У `matrices/` лише `meditation-to-book-iv.md`

## Completeness

- [x] README, RULES, INDEX, LIVING_SPEC
- [x] `book-analyses/book-iv-reference.md`
- [x] matrix, provenance, TODO, gates
- [x] INDEX pointers у Analyze-1…4

## Commands

```bash
test -f Analyze-5/book-analyses/book-iv-reference.md
test "$(ls Analyze-5/matrices | wc -l)" -eq 1
bash Analyze-5/verification/deny-originals.sh
bash Analyze-5/verification/deny-foreign-workspace.sh
git status --porcelain -- 'Manifesto etc' 'Meditation_About'
```

## Result

```text
deny-originals.sh → OK
deny-foreign-workspace.sh → OK
matrices count = 1
Manifesto etc / Meditation_About → unchanged
```

**Verdict:** PASS
