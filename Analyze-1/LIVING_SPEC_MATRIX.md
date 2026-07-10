# Living Spec Matrix — Analyze-1

Матриця відповідності: **первісне ТЗ / джерело → канонічна книга → аналітичний артефакт → перевірка**.

Оновлювати при кожному новому аналітичному артефакті в Analyze-*.  
**Не** змінює первісні документи.  
**Джерела лише з цього репозиторію.**

| ID | Джерело (immutable) | Канон Manifesto | Аналітика Analyze-1 | Тест / перевірка | Статус |
|----|---------------------|-----------------|---------------------|------------------|--------|
| LS-001 | `Manifesto etc/Book 0 — … v0.1.md` | Book 0 | `book-analyses/book-0-foundations.md` | Conformance B0-001…005 (спека, не код) | mapped |
| LS-002 | `Meditation_About/1`…`62a` | Book 0 + SCD Historical | `matrices/meditation-to-book0.md` | CHECKLIST §matrices | mapped |
| LS-003 | `Manifesto etc/AIRA Specification Control Document v0.1.md` | SCD | `provenance/corpus-map.md` | CHECKLIST §immutability | mapped |
| LS-004 | Book 0 §7 + `Meditation_About/58` | Book 0 SST | `matrices/meditation-to-book0.md`, TF-001 | Provenance SST у репо | mapped |
| LS-005 | Book I–V | Books I–V | Analyze-2 (Book I); решта Analyze-3+ | Analyze-2 CHECKLIST | Book I done |
| LS-006 | Conformance / Schema / MVP Issue Set | Ecosystem | provenance only | runtime N/A | deferred |

## Формула стандарту (Evidence з SCD §15)

```text
AIRA Standard = Book 0 + Book I + Book II + Book III + Conformance Tests
AIRA Ecosystem = Standard + Book IV + Book V + RFC + Research Promotion
```

## Правило оновлення

1. Нова фіча/аналіз → новий рядок LS-xxx.  
2. Посилання на MR/commit — у колонці «Тест / перевірка» коли зʼявиться код.  
3. Зміна канону — лише через RFC-чернетку в Analyze-*, не edit Book *.
