# Verification Checklist — Analyze-1

## Immutability

- [x] `git status` / `git diff` не містить змін у `Manifesto etc/`
- [x] `git status` / `git diff` не містить змін у `Meditation_About/`
- [x] Нові/змінені аналітичні файли лише під `Analyze-1/` (і `.omx/` за потреби)
- [x] Скрипт `verification/deny-originals.sh` проходить

## Repo boundary

- [x] Немає відсилок до зовнішніх робочих просторів у `Analyze-1/`
- [x] `verification/deny-foreign-workspace.sh` проходить
- [x] У `matrices/` лише `meditation-to-book0.md`

## Completeness Analyze-1

- [x] `README.md` з immutability + межами репо
- [x] `RULES.md`, `INDEX.md`, `LIVING_SPEC_MATRIX.md`
- [x] `book-analyses/book-0-foundations.md`
- [x] `matrices/meditation-to-book0.md`
- [x] `provenance/corpus-map.md` (бакети SCD §11.1 / §11.2)
- [x] `todo/TODO_FIXME.md`

## Commands

```bash
# from repository root
test -d Analyze-1
test -f Analyze-1/README.md
test -f Analyze-1/matrices/meditation-to-book0.md
test "$(ls Analyze-1/matrices | wc -l)" -eq 1
bash Analyze-1/verification/deny-originals.sh
bash Analyze-1/verification/deny-foreign-workspace.sh
git status --porcelain -- 'Manifesto etc' 'Meditation_About'
```

## Result (remediation: repo boundary)

```text
deny-originals.sh → OK
deny-foreign-workspace.sh → OK
cross-repo provenance removed from Analyze-1
```

**Verdict:** PASS
