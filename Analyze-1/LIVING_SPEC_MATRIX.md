# Living Spec Matrix — Analyze-1

Матриця відповідності: **первісне ТЗ / джерело → канонічна книга → аналітичний артефакт → перевірка**.

Оновлювати при кожному новому аналітичному артефакті в Analyze-*.  
**Не** змінює первісні документи.

| ID | Джерело (immutable) | Канон Manifesto | Аналітика Analyze-1 | Тест / перевірка | Статус |
|----|---------------------|-----------------|---------------------|------------------|--------|
| LS-001 | `Manifesto etc/Book 0 — … v0.1.md` | Book 0 | `book-analyses/book-0-foundations.md` | Conformance B0-001…005 (спека, не код) | mapped |
| LS-002 | `../MIND/Meditation_About/sTOR-*.txt` | Book 0 principles A0–A7, §11–12 | `matrices/stor-to-book0.md` | verification/CHECKLIST §matrices | mapped |
| LS-003 | `Meditation_About/1`…`62a` | Book 0 + SCD Historical | `matrices/aira-meditation-to-book0.md` | CHECKLIST §matrices | mapped |
| LS-004 | `Manifesto etc/AIRA Specification Control Document v0.1.md` | SCD | `provenance/corpus-map.md` | CHECKLIST §immutability | mapped |
| LS-005 | Book 0 §7 Solution Space | Book 0 | `todo/TODO_FIXME.md` TF-001 | Gap: SST у sTOR відсутній | open |
| LS-006 | Book I–V | Books I–V | — | Analyze-2+ | pending |
| LS-007 | Conformance / Schema / MVP Issue Set | Ecosystem | provenance only | runtime N/A | deferred |

## Формула стандарту (Evidence з SCD §15)

```text
AIRA Standard = Book 0 + Book I + Book II + Book III + Conformance Tests
AIRA Ecosystem = Standard + Book IV + Book V + RFC + Research Promotion
```

## Правило оновлення

1. Нова фіча/аналіз → новий рядок LS-xxx.  
2. Посилання на MR/commit — у колонці «Тест / перевірка» коли зʼявиться код.  
3. Зміна канону — лише через RFC-чернетку в Analyze-*, не edit Book *.
