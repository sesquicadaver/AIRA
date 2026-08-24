# AIRA Specification Control Document v0.1

```text
Type: Normative / Governance
Scope: AIRA Standard Corpus
Status: Draft v0.1
Depends on:
  - Book 0 — Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
  - Book IV — Reference Implementation
  - Book V — Research Track
Exports:
  - Standard Corpus Rules
  - Document Status Model
  - Change Control Process
  - RFC Admission Rules
  - Conformance Matrix
  - Source Pruning Rules
```

---

# 1. Purpose

Цей документ визначає, як підтримувати AIRA як **стандарт**, а не як набір еволюційних нотаток.

Після завершення Book 0–V активний корпус має містити лише документи, які виконують одну з функцій:

```text
define foundations
define core
define protocols
define CSU contracts
define reference implementation
define research boundary
define conformance
```

Усе інше переходить у `Superseded`, `Historical` або `Research Archive`.

---

# 2. Corpus Structure

Канонічний корпус AIRA складається з шести основних книг:

```text
Book 0 — Mathematical & Conceptual Foundations
Book I — Core Architecture & ABI
Book II — Protocol Specifications
Book III — CSU & ABI Contracts
Book IV — Reference Implementation
Book V — Research Track
```

Book 0–III є нормативними. Book IV є референсним, але не нормативним. Book V є research/non-normative. Це узгоджується з уже сформованою структурою, де Book I визначає ядро, Book II — протоколи, Book III — CSU-контракти, а Book IV лише демонструє мінімальну реалізацію.     

---

# 3. Normative Status

## 3.1 Normative

Нормативні документи визначають сумісність реалізацій.

```text
Book 0
Book I
Book II
Book III
Specification Control Document
Conformance Test Specification
```

Якщо реалізація порушує ці документи, вона не є AIRA-compatible.

---

## 3.2 Reference

Reference-документи описують рекомендований спосіб реалізації, але не є єдино допустимими.

```text
Book IV
Reference Node Design
Reference CLI
Reference Storage
Reference CSU Set
Reference Test Harness
```

Реалізація може відрізнятися від Book IV і залишатися сумісною, якщо вона виконує Book 0–III.

---

## 3.3 Research

Research-документи не визначають сумісність.

```text
Book V
Exploratory Resolution
Cognitive Field
Distributed Scientific Method
Capability Algebra
Self-Organizing Constellations
Advanced PHM
Goal Compiler
```

Research може впливати на майбутні RFC, але не змінює Reference автоматично.

---

# 4. Document Status Model

Кожне джерело AIRA повинно мати один статус.

```text
Canonical
Reference
Research
Superseded
Historical
Rejected
```

## 4.1 Canonical

Чинна нормативна специфікація.

## 4.2 Reference

Чинний приклад реалізації.

## 4.3 Research

Активний дослідницький напрям.

## 4.4 Superseded

Повністю перекритий пізнішим документом.

## 4.5 Historical

Збережений лише для історії еволюції.

## 4.6 Rejected

Ідея явно відхилена.

---

# 5. Supersession Rule

Документ вважається `Superseded`, якщо виконуються всі умови:

```text
1. Він не вводить чинну фундаментальну сутність.
2. Він не містить чинного інваріанта.
3. Його протокольні вимоги перекриті Book II.
4. Його CSU-вимоги перекриті Book III.
5. Його реалізаційні ідеї перекриті Book IV або перенесені в Book V.
```

Фізично такі документи можна прибрати з активного корпусу після перевірки, що всі унікальні фрагменти перенесені.

---

# 6. Active Corpus Rule

Активний корпус не повинен містити дублікати.

Якщо два документи описують одну й ту саму сутність, активним залишається той, який має вищий пріоритет:

```text
Book 0–III
↓
Specification Control Document
↓
Conformance Specification
↓
Book IV
↓
Book V
↓
Research Notes
↓
Historical Drafts
```

---

# 7. Change Admission Rule

Будь-яка зміна AIRA проходить через RFC.

RFC повинна відповісти на питання:

```text
1. Яку проблему вирішує зміна?
2. До якого рівня вона належить?
3. Чи змінює вона Core Ontology?
4. Чи змінює вона ABI?
5. Чи змінює вона Protocol?
6. Чи змінює вона CSU Contract?
7. Чи можна реалізувати її як CSU / Protocol Extension / Algorithm / Artifact?
8. Який вимірюваний ефект вона дає?
9. Який rollback path?
10. Які conformance tests потрібні?
```

---

# 8. RFC Classes

## RFC-A — Architecture RFC

Змінює Book 0 або Core Ontology.

Допускається лише якщо доведено:

```text
logical contradiction
incomplete ontology
unrepresentable task class
broken invariant
```

## RFC-B — Core RFC

Змінює Book I або Stable ABI.

Вимагає major version.

## RFC-C — Protocol RFC

Змінює Book II.

Може бути backward-compatible або breaking.

## RFC-D — CSU RFC

Змінює Book III.

Додає або уточнює CSU role contract.

## RFC-E — Reference RFC

Змінює Book IV.

Не впливає на сумісність.

## RFC-R — Research RFC

Додає або формалізує Research Track.

---

# 9. Compatibility Rule

Дві реалізації є сумісними, якщо вони:

```text
support Book 0 ontology;
support Book I ABI and object model;
support required Book II protocols for declared conformance level;
support required Book III CSU contracts;
produce valid Verified Result Artifact;
preserve Artifact immutability;
preserve Event causality;
enforce Policy Gate;
emit Failure Evidence.
```

Внутрішні алгоритми можуть повністю відрізнятися.

---

# 10. Conformance Profiles

## AIRA-C0 — Local Core

```text
Book 0
Book I
local Event Runtime
local Artifact Runtime
Policy Gate
Invariant Checker
```

## AIRA-C1 — Local Operational Node

```text
C0
+ Context CSU
+ Reduction CSU
+ Execution CSU
+ Verification CSU
+ Evidence CSU
```

## AIRA-C2 — Protocol-Compatible Node

```text
C1
+ Event Protocol
+ Artifact Protocol
+ Identity Protocol
+ Discovery Protocol
```

## AIRA-C3 — Federated Node

```text
C2
+ Federation Protocol
+ Capability Advertisement
+ CRP
```

## AIRA-C4 — Settlement-Compatible Node

```text
C3
+ Settlement / Audit Protocol
```

## AIRA-C5 — Research-Capable Node

```text
C4
+ Evolution CSU
+ Research CSU
+ Artifact Promotion Pipeline
```

C5 не є обов’язковим для базової сумісності.

---

# 11. Source Pruning Policy

Активні джерела повинні бути мінімізовані.

## 11.1 Immediate Removal Candidates

Документи, повністю перекриті Book 0–V, переводяться в `Superseded`.

З активного корпусу можна прибирати:

```text
1, 2, 5, 6, 7, 8, 13, 16, 17, 19, 20, 21, 23, 27, 30, 37, 40, 41, 43, 46, 49, 50, 55, 56, 57, 61, 62
```

Причина: вони замінені Book 0–V, Canonical Terminology, Core ABI, CSU model, Protocol model або Research Track.

## 11.2 Merge Before Removal

Перед вилученням потрібно перенести залишки з:

```text
3, 4, 9, 10, 11, 15, 22, 24, 25, 26, 31, 32, 32A, 33, 34, 35, 36, 38, 39, 51, 52, 54, 58, 59, 60
```

Ці документи не повинні лишатися активними як специфікації, але містять фрагменти для Book II–V.

---

# 12. Canonical Artifact Rule

Кожне завершення роботи зі специфікацією повинно породжувати Artifact:

```text
Canonical Specification Artifact
Reference Implementation Artifact
Research Artifact
Supersession Artifact
Conformance Artifact
```

Не повинно бути незафіксованих рішень.

---

# 13. Versioning

AIRA використовує незалежне версіонування:

```text
AIRA Standard Version
Book Version
ABI Version
Protocol Version
CSU Contract Version
Artifact Schema Version
Conformance Profile Version
Research Track Version
```

Зміна Book IV або Book V не змінює AIRA Standard Version.

Зміна Book I ABI зазвичай потребує major version.

---

# 14. Stability Rule

```text
Book 0 — змінюється майже ніколи.
Book I — змінюється рідко.
Book II — змінюється контрольовано.
Book III — розширюється через нові CSU contracts.
Book IV — еволюціонує вільно.
Book V — еволюціонує без обмежень, але без впливу на Reference.
```

---

# 15. Final Standard Formula

```text
AIRA Standard =
    Book 0
  + Book I
  + Book II
  + Book III
  + Conformance Tests
```

```text
AIRA Ecosystem =
    AIRA Standard
  + Book IV
  + Book V
  + RFC Process
  + Research Promotion Pipeline
```

---

# 16. Status

Specification Control Document v0.1 фіксує перехід AIRA з концептуального проєкту в керований стандарт.

Наступний нормативний документ:

```text
AIRA Conformance Test Specification v0.1
```

Його задача:

```text
перевірити Book 0 invariants;
перевірити Book I object / ABI compatibility;
перевірити Book II protocol envelopes;
перевірити Book III CSU contracts;
перевірити Verified Result Artifact;
перевірити failure → evidence behavior;
перевірити Operational / Evolution separation.
```

---

# Appendix A — Repository implementation pointer (non-normative)

Цей appendix **не** змінює нормативний корпус. Вказує на живий reference tree у репозиторії.

| Artifact | Path |
|----------|------|
| Execution queue | [`QUEUE.md`](../QUEUE.md) — Phase G `#120`–`#146` OPEN |
| Implementation matrix | [`docs/implementation-status.md`](../docs/implementation-status.md) |
| Documentation index | [`docs/README.md`](../docs/README.md) |
| Phase plans | [`docs/phase-c-plan.md`](../docs/phase-c-plan.md) … [`docs/phase-g-plan.md`](../docs/phase-g-plan.md) |
| Conformance runners | `crates/aira-conformance/`; CI [`docs/ci-governance.md`](../docs/ci-governance.md) |
| Anti-mission | README §«What AIRA is not»; [`docs/phase-g-plan.md`](../docs/phase-g-plan.md) §3 |

**Оновлено:** 2026-08-24 (Phase F DONE; Phase G OPEN).
