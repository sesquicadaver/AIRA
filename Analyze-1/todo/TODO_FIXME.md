# TODO_FIXME — Analyze-1

Список виявлених прогалин, напруг і ризиків.  
**Не** є правками первісних документів.

| ID | Severity | Область | Опис | Evidence | Пропонована дія (лише Analyze/RFC) |
|----|----------|---------|------|----------|-------------------------------------|
| TF-001 | HIGH | Book 0 §7 | Solution Space Theory відсутня в MIND sTOR; залежить від Meditation 58 | matrices/* | Зафіксовано; при RFC-A тримати 58 як provenance |
| TF-002 | LOW | Термінологія Goal | Рання частина doc 62 піднімає Goal над Problem; **кінець doc 62** явно виносить Goal з мікроядра → Goal Compiler Driver; ядро лишає Problem Statement / Intent / Artifact / Event / Capsule | Meditation_About/62 (фінал); Book 0 §3.2 | Не конфлікт канону — закрито Evidence. Лише стежити за цитатами «Goal first» без фінальної поправки |
| TF-003 | MEDIUM | SCD §11 vs immutability | SCD радить prune Historical drafts; користувач заборонив змінювати originals | SCD §11; user rule | Prune лише після явного дозволу; доти — статуси в provenance |
| TF-004 | MEDIUM | Conformance | B0 тести в спекі; коду harness немає | Conformance Spec §7 | Відкласти до runtime циклу |
| TF-005 | LOW | Book 0 §4 Differentiate | Слабке покриття в sTOR і ранніх AIRA docs | stor-to-book0 | Analyze-2: трасування в Book I/III Epistemic/Human CSU |
| TF-006 | LOW | Dual Meditation | Дві теки «Meditation_About» (AIRA vs MIND) плутають provenance | README Analyze-1 | Завжди кваліфікувати шлях |
| TF-007 | INFO | Books I–V | Не розібрані в Analyze-1 | INDEX | Analyze-2+ |
| TF-008 | INFO | Living Spec ↔ CI | Немає автогенерації матриці в CI | LIVING_SPEC_MATRIX | Пізніше, коли зʼявиться CI |

## Anti-stub note

У первісних Manifesto docs цього циклу stubs/`pass`/`return None` не шукались як код — корпусу runtime ще немає. Перевірка anti-stub для коду — у майбутньому implementation циклі.
