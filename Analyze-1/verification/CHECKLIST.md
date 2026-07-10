# Verification Checklist — Analyze-1

## Immutability

- [x] `git status` / `git diff` не містить змін у `Manifesto etc/`
- [x] `git status` / `git diff` не містить змін у `Meditation_About/`
- [x] Нові файли лише під `Analyze-1/` (і службові `.omx/` за потреби autopilot)
- [x] Скрипт `verification/deny-originals.sh` проходить

## Completeness Analyze-1

- [x] `README.md` з правилом immutability
- [x] `RULES.md`, `INDEX.md`, `LIVING_SPEC_MATRIX.md`
- [x] `book-analyses/book-0-foundations.md`
- [x] `matrices/stor-to-book0.md`
- [x] `matrices/aira-meditation-to-book0.md`
- [x] `provenance/corpus-map.md` (бакети SCD §11.1 / §11.2)
- [x] `todo/TODO_FIXME.md`

## Commands

```bash
# from repository root
test -d Analyze-1
test -f Analyze-1/README.md
bash Analyze-1/verification/deny-originals.sh
git status --porcelain -- 'Manifesto etc' 'Meditation_About'
```

## Result (Ralph 2026-07-10, rev 2)

```text
FILES_OK
deny-originals.sh → OK
git status originals → empty
Review cycle 1 findings addressed in Analyze-1 only
```

**Verdict:** PASS
