# Матриця: Governance → MVP → Books

| Артефакт | Роль | Залежить від | Виконує / перевіряє | MVP touch |
|----------|------|--------------|---------------------|-----------|
| SCD | Corpus rules + Standard formula | Books 0–V | Що є Standard | Freeze M0 |
| Conformance Spec | Compatibility oracle | Book 0–III + SCD | B0–B3, OP, SEP, SEC… | C0/C1 harness |
| Schema Pack | Data contracts | Books + Conformance | JSON schemas by profile | M1 / Issues #9–21 |
| RFC Process | Change control | SCD + Conformance + Schema | Gates A…X | Post-MVP evolution |
| MVP Roadmap | Milestones M0–M13 | Book IV + Schema + Conf | Build order | All M* |
| Bootstrap | Repo skeleton | Roadmap | crates/specs/CI | Issues #1–5 |
| Issue Set | Backlog #1–#80 | Bootstrap + Roadmap | PR-001…016 | Freeze to #80 |
| MVP PRD | Product acceptance | Roadmap + Conf | FR/NFR/release | Issue #80 |

## Документний «next» ланцюг (Status sections)

```text
SCD → Conformance → Schema Pack → RFC → MVP Roadmap → Bootstrap → Issue Set → MVP PRD
```

(У репо всі вже присутні; порядок = рекомендована логіка читання, не git history.)

## Поточний стан репо vs Bootstrap

| Bootstrap expect | У цьому репо зараз |
|------------------|-------------------|
| `specs/` Books | `Manifesto etc/` (immutable originals) |
| `crates/` | відсутні |
| `schemas/` JSON | лише Schema Pack markdown |
| `aira` CLI | відсутній |
| Analyze-* | аналітичний шар (поза Bootstrap layout) |

**Висновок:** Analyze-1…7 = corpus analysis complete; runtime = новий цикл від Issue #1.
