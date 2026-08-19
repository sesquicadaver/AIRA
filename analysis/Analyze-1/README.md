# Analyze-1 — аналітичний шар первісного проєкту AIRA

**Статус:** Active  
**Цикл:** Analyze-1  
**Мова:** українська

## Призначення

Ця тека — **єдине місце** для спостережень, аналітики, матриць відповідності, TODO/FIXME і висновків щодо первісного корпусу AIRA.

## Жорсткі правила

### Immutability первісних документів

```text
ЗАБОРОНЕНО змінювати, перейменовувати, переміщувати або «причісувати»:

  Manifesto etc/**
  Meditation_About/**
```

### Межі робочого простору

```text
Працювати ВИКЛЮЧНО з матеріалами цього репозиторію AIRA.
Заборонено підмішувати зовнішні проєкти, «спільну абетку» чи чужі Meditation_About.
Джерела істини: Manifesto etc/ + Meditation_About/ (+ Analyze-N для аналітики).
```

## Первісні джерела (не чіпати)

| Шлях | Роль |
|------|------|
| `Manifesto etc/` | Канонічний корпус Book 0–V + governance/MVP |
| `Meditation_About/` | Еволюційні роздуми AIRA (docs 1–62+) |

## Структура Analyze-1

```text
Analyze-1/
  README.md
  INDEX.md
  LIVING_SPEC_MATRIX.md
  RULES.md
  book-analyses/book-0-foundations.md
  matrices/meditation-to-book0.md
  provenance/corpus-map.md
  todo/TODO_FIXME.md
  verification/CHECKLIST.md
  verification/deny-originals.sh
  verification/deny-foreign-workspace.sh
```

## Наступні цикли

- Analyze-2: Book I  
- Analyze-3: Book II–III  
- Runtime / Cargo — лише після окремого рішення
