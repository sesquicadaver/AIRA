# TODO_FIXME — Analyze-1

Список виявлених прогалин, напруг і ризиків.  
**Не** є правками первісних документів.  
Evidence — лише з цього репозиторію.

| ID | Severity | Область | Опис | Evidence | Пропонована дія (лише Analyze/RFC) |
|----|----------|---------|------|----------|-------------------------------------|
| TF-001 | INFO | Book 0 §7 | SST канонізовано в Book 0; прямий предок у репо — `Meditation_About/58` | Book 0 §7; Meditation_About/58 | Provenance зафіксовано в матриці; RFC-A лише при зміні §7 |
| TF-002 | LOW | Термінологія Goal | Рання частина doc 62 піднімає Goal над Problem; **кінець doc 62** виносить Goal з мікроядра → Goal Compiler Driver | Meditation_About/62 (фінал); Book 0 §3.2 | Стежити за цитатами «Goal first» без фінальної поправки |
| TF-003 | MEDIUM | SCD §11 vs immutability | SCD радить prune Historical drafts; користувач заборонив змінювати originals | SCD §11; user rule | Prune лише після явного дозволу; доти — статуси в provenance |
| TF-004 | MEDIUM | Conformance | B0 тести в спекі; коду harness немає | Conformance Spec §7 | Відкласти до runtime циклу |
| TF-005 | LOW | Book 0 §4 Differentiate | Слабке явне покриття в ранніх Meditation docs; сильніше в пізній епістеміці | Meditation_About (ранні vs 58–62); Book 0 §4 | Analyze-2: трасування в Book I/III |
| TF-006 | INFO | Books I–V | Не розібрані в Analyze-1 | INDEX | Analyze-2+ |
| TF-007 | INFO | Living Spec ↔ CI | Немає автогенерації матриці в CI | LIVING_SPEC_MATRIX | Пізніше, коли зʼявиться CI |
| TF-008 | HIGH | Workspace hygiene | Раніше в Analyze-1 потрапили відсилки до чужого робочого простору | цей цикл remediation | Видалено; RULES §0 + `deny-foreign-workspace.sh` |

## Anti-stub note

У первісних Manifesto docs цього циклу stubs/`pass`/`return None` не шукались як код — корпусу runtime ще немає.
