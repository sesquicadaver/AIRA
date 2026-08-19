# Verification Checklist — Analyze-2

## Immutability

- [x] Немає змін у `Manifesto etc/`
- [x] Немає змін у `Meditation_About/`
- [x] Нові файли під `Analyze-2/` (+ pointer у Analyze-1 INDEX)

## Repo boundary

- [x] `deny-foreign-workspace.sh` PASS
- [x] У `matrices/` лише `meditation-to-book-i.md`

## Completeness

- [x] README, RULES, INDEX, LIVING_SPEC
- [x] `book-analyses/book-i-core-abi.md` (Evidence-labeled digest §1–25)
- [x] `matrices/meditation-to-book-i.md`
- [x] provenance, TODO, verification scripts

## Commands

```bash
# from repository root
test -f Analyze-2/book-analyses/book-i-core-abi.md
test -f Analyze-2/matrices/meditation-to-book-i.md
test "$(ls Analyze-2/matrices | wc -l)" -eq 1
bash Analyze-2/verification/deny-originals.sh
bash Analyze-2/verification/deny-foreign-workspace.sh
git status --porcelain -- 'Manifesto etc' 'Meditation_About'
```

## Result (Ralph rev 2)

```text
deny-originals.sh → OK
deny-foreign-workspace.sh → OK
matrices count = 1
Manifesto etc / Meditation_About → unchanged
Evidence labels added to book-i-core-abi.md
```

**Verdict:** PASS
