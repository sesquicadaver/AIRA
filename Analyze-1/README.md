# Analyze-1 — аналітичний шар первісного проєкту AIRA

**Статус:** Active  
**Цикл:** Analyze-1 (autopilot 2026-07-10)  
**Мова:** українська

## Призначення

Ця тека — **єдине місце** для спостережень, аналітики, матриць відповідності, TODO/FIXME і висновків щодо первісного корпусу AIRA.

## Жорстке правило (immutability)

```text
ЗАБОРОНЕНО змінювати, перейменовувати, переміщувати або «причісувати»
первісні документи:

  Manifesto etc/**
  Meditation_About/**
```

Первісні файли — **read-only джерела істини** для аналізу.  
Будь-які правки, узагальнення, критика, Living Spec і пропозиції RFC живуть **лише** тут (або в наступних `Analyze-N/`).

## Первісні джерела (не чіпати)

| Шлях | Роль |
|------|------|
| `Manifesto etc/` | Канонічний корпус Book 0–V + governance/MVP |
| `Meditation_About/` | Еволюційні роздуми AIRA (docs 1–62+) |
| `../MIND/Meditation_About/` (зовнішнє) | sTOR-1…33 AI OS — концептуальний предок патернів |

## Структура Analyze-1

```text
Analyze-1/
  README.md
  INDEX.md
  LIVING_SPEC_MATRIX.md
  RULES.md
  book-analyses/book-0-foundations.md
  matrices/stor-to-book0.md
  matrices/aira-meditation-to-book0.md
  provenance/corpus-map.md
  todo/TODO_FIXME.md
  verification/CHECKLIST.md
  verification/deny-originals.sh   ← soft gate: originals незмінені
```

## Наступні цикли (поза scope Analyze-1)

- Analyze-2: Book I  
- Analyze-3: Book II–III  
- Runtime / Cargo — лише після окремого рішення
