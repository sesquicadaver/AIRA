# AIRA — аналітичний шар

Тека для результатів дослідження, розбору специфікацій і висновків Analyze-циклів.

## Структура

| Шлях | Роль |
|------|------|
| `Analyze-N/` | Один Analyze-цикл (deep-interview → ralplan → impl/review → close) |
| `Analyze-N/book-analyses/` | Розбір книг / корпусу |
| `Analyze-N/provenance/` | Scope, handoff, ADR |
| `Analyze-N/verification/` | CODE_REVIEW, UltraQA, deny-* gates |
| `Analyze-N/todo/TODO_FIXME.md` | Provenance дефектів (не канон backlog) |

## Правила

1. **Не редагувати** `Manifesto etc/` і `Meditation_About/` — лише цитувати.
2. **Канон backlog** — [`QUEUE.md`](../QUEUE.md) у корені репозиторію. Phase C: [`docs/phase-c-plan.md`](../docs/phase-c-plan.md).
3. **Новий цикл** — нова тека `analysis/Analyze-N/` (наступний вільний номер); див. [`Analyze-1/RULES.md`](Analyze-1/RULES.md).
4. **Evidence vs Inference** — кожне твердження з джерелом у цьому репо або явно як inference.

## Зв'язок з кодом

Rust-крейти та `docs/` можуть посилатися на Analyze-N **як provenance** (коментарі, Living Spec). Імплементація живе в `crates/`, `docs/`, `deploy/` — не в цій теці.
