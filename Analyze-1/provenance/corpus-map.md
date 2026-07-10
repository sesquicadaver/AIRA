# Карта корпусу AIRA (provenance)

**Цикл:** Analyze-1 (rev 2 після review COMMENT)  
**Правило:** originals не змінюються; статуси нижче — **аналітична** класифікація за SCD.

## 1. Канонічний Manifesto (`Manifesto etc/`)

| Документ | SCD-клас (Evidence SCD §3) | Примітка |
|----------|----------------------------|----------|
| Book 0 | Normative | Foundations |
| Book I | Normative | Core ABI |
| Book II | Normative | Protocols |
| Book III | Normative | CSU contracts |
| Book IV | Reference | Не єдино допустима реалізація |
| Book V | Research | Non-normative |
| Specification Control Document | Normative / Governance | Corpus rules |
| Conformance Test Specification | Normative | B0–B3, OP, SEP tests |
| RFC Template & Change Process | Governance | RFC-A…R |
| Schema Pack | Supporting | Schemas |
| MVP PRD / Roadmap / Bootstrap / Issue Set | Product / Engineering | Не Standard formula |

## 2. Meditation_About (AIRA repo) — бакети SCD §11

**Evidence:** `Manifesto etc/AIRA Specification Control Document v0.1.md` §11.1–11.2.  
**Constraint:** Analyze-1 **не** prune/delete ці файли (user immutability > SCD prune suggestion).

### 2.1 Immediate Removal Candidates (SCD §11.1)

```text
1, 2, 5, 6, 7, 8, 13, 16, 17, 19, 20, 21, 23, 27, 30, 37, 40, 41, 43, 46, 49, 50, 55, 56, 57, 61, 62
```

Аналітичний статус: `Superseded candidates` (SCD). Фізично лишаються в `Meditation_About/`.

### 2.2 Merge Before Removal (SCD §11.2)

```text
3, 4, 9, 10, 11, 15, 22, 24, 25, 26, 31, 32, 32A, 33, 34, 35, 36, 38, 39, 51, 52, 54, 58, 59, 60
```

Аналітичний статус: `Merge-before-removal` — фрагменти цінні для Book II–V / Book 0 provenance (напр. 58 SST).

### 2.3 Поза списками SCD §11 (у репо є)

| Файл | Примітка (Inference) |
|------|----------------------|
| 12, 14, 18, 28, 29, 42, 44, 45, 47.1–47.3, 48, 53 | Не в §11.1/11.2 — окрема класифікація; **42** критичний для §11 Book 0 (microkernel) |
| 32a, 37a, 62a | Суфіксні уточнення; 32A згаданий у §11.2 як 32A |

## 3. Зовнішнє MIND Meditation_About (sTOR)

Концептуальний предок Policy/Object/Verify/Promote/Freeze.  
Не є файлами цього git-репо. Шлях: sibling `../MIND/Meditation_About/` (або `$MIND_ROOT/Meditation_About`).  
Див. `matrices/stor-to-book0.md`.

## 4. Ієрархія пріоритету (Evidence SCD §6)

```text
Book 0–III
→ SCD
→ Conformance
→ Book IV
→ Book V
→ Research Notes
→ Historical Drafts (Meditation_About)
```

## 5. Analyze-1 у цій ієрархії

`Analyze-1/` = **аналітичний шар**, не canonical spec.  
Не піднімає пріоритет над Book 0–III. Може пропонувати RFC-чернетки.
