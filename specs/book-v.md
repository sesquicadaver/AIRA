# Book V — Research Track v0.1

```text
Type: Research / Non-Normative
Scope: AIRA Research Track
Status: Draft v0.1
Depends on:
  - Book 0 — Mathematical & Conceptual Foundations
  - Book I — Core Architecture & ABI
  - Book II — Protocol Specifications
  - Book III — CSU & ABI Contracts
  - Book IV — Reference Implementation
Exports:
  - Research Governance Model
  - Research Artifact Model
  - Research CSU Profiles
  - Exploratory Resolution Model
  - Cognitive Evolution Model
  - Promotion Pipeline
  - Research Safety Boundaries
```

Book V описує **експериментальний контур AIRA**. Він не є обов’язковим для сумісної реалізації. Сумісність визначається Book 0–III; Book IV описує референсну реалізацію. Book V визначає, як дослідницькі механізми можуть існувати, не руйнуючи ядро, протоколи, CSU-контракти й Operational Plane.     

---

# 1. Purpose

Book V існує для довгострокового розвитку AIRA.

Його задача:

```text
досліджувати нові способи зменшення невизначеності;
генерувати нові гіпотези;
перевіряти нові алгоритми;
покращувати майбутні Resolution;
розвивати Knowledge / Evidence / Artifact ecosystem;
не впливати напряму на Operational Plane.
```

Book V відповідає за те, що раніше описувалося як:

```text
Exploratory Resolution
Cognitive Evolution
Distributed Scientific Method
Self-Organizing Constellations
Capability Algebra
Advanced PHM
Knowledge Field
Goal Compiler
Collective Wisdom
Research CSU
Background Experiments
```

---

# 2. Non-Goals

Book V **не визначає**:

```text
Core Object Model
Stable ABI
Protocol compatibility
mandatory CSU contracts
Reference Implementation behavior
required ML model
required optimizer
required blockchain
required global knowledge graph
```

Жоден компонент Book V не є умовою AIRA-сумісності.

---

# 3. Research Boundary

Research Track **MUST NOT**:

```text
змінювати Core;
обходити Stable ABI;
обходити Policy Gate;
змінювати Artifact in-place;
створювати Operational Artifact напряму;
виконувати прихований normative collapse;
вимагати змін Book 0–III без RFC;
впливати на Operational Plane без Artifact Promotion.
```

Research може створювати:

```text
Evidence Artifact
Hypothesis Artifact
Research Artifact
Best Current Hypothesis Artifact
Negative Result Artifact
Open Research Artifact
Artifact Promotion Candidate
```

---

# 4. Research Plane Position

AIRA має два незалежні контури.

## 4.1 Operational Plane

```text
Problem Statement
↓
Interpret
↓
Reduce
↓
Differentiate
↓
Organize
↓
Materialize
↓
Verify
↓
Verified Result Artifact
```

## 4.2 Research / Evolution Plane

```text
Evidence
↓
Aggregate
↓
Discover
↓
Experiment
↓
Validate
↓
Promote
↓
Operational Artifact Candidate
```

Research Plane працює асинхронно.

Operational Plane **не чекає** Research Plane.

Research Plane покращує майбутні рішення, але не блокує поточні.

---

# 5. Research Admission Rule

Ідея потрапляє до Book V, якщо вона:

```text
не потрібна для мінімальної сумісної AIRA;
може покращити якість, вартість, швидкість або надійність;
не має достатніх доказів для Reference;
потребує експериментів;
може бути реалізована як CSU / Algorithm / Artifact / Protocol Extension.
```

Якщо ідея потребує нової фундаментальної сутності, вона не приймається одразу до Reference. Спочатку вона проходить Research Track.

---

# 6. Research Artifact Model

Кожен Research Artifact **MUST** мати:

```yaml
artifact_id: string
artifact_type: ResearchArtifact
research_domain: string
hypothesis_refs: [artifact_ref]
evidence_refs: [evidence_ref]
counter_evidence_refs: [evidence_ref]
method_ref: artifact_ref
experiment_refs: [artifact_or_event_ref]
result_status: enum
confidence: number
scope: scope_descriptor
risk_descriptor: object
policy_refs: [policy_ref]
provenance_refs: [artifact_or_event_ref]
created_at: timestamp
signature: signature
```

## 6.1 result_status

```text
proposed
running
replicated
failed
contradicted
validated
promoted_candidate
deprecated
```

---

# 7. Research Lifecycle

```text
Proposal
↓
Hypothesis
↓
Experiment Design
↓
Execution
↓
Evidence Collection
↓
Validation
↓
Replication
↓
Promotion Candidate
↓
Reference RFC або Archive
```

Жоден Research Artifact не може бути використаний Operational Plane напряму.

---

# 8. Promotion Pipeline

Research → Operational відбувається тільки через promotion.

```text
Research Artifact
↓
Evidence Review
↓
Epistemic Validation
↓
Compatibility Check
↓
Safety Check
↓
Cost/Benefit Evidence
↓
Canary Deployment
↓
Reference RFC
↓
Optional / Reference / Rejected
```

## 8.1 Promotion criteria

Research може перейти до Reference лише якщо доведено:

```text
вимірюване зменшення total cost;
сумісність з Book 0–III;
відсутність зміни Core;
наявність conformance tests;
наявність failure semantics;
наявність rollback path;
відсутність прихованого normative collapse.
```

---

# 9. Research Budget Model

Research Plane **MUST** мати окремий бюджет.

```text
Discovery Budget
Experiment Budget
Validation Budget
Promotion Budget
Energy Budget
Risk Budget
Human Review Budget
```

Research **MUST NOT** витісняти Operational Plane.

Допустимі джерела ресурсів:

```text
idle capacity
voluntary research pools
specialized research federations
low-priority background compute
scheduled research windows
donated compute
```

---

# 10. Exploratory Resolution

Exploratory Resolution — рівень ескалації після детермінованих методів.

```text
L0 Ready Solution
L1 Verified Knowledge
L2 Deterministic Resolution
L3 Exploratory Resolution
L4 Cognitive Evolution
```

## 10.1 L3 — Exploratory Resolution

Мета:

```text
знайти новий спосіб зменшити Solution Uncertainty;
згенерувати нову гіпотезу;
знайти альтернативну область Solution Field;
виявити неочевидний зв’язок між доменами.
```

L3 може використовувати:

```text
ML
LLM ensembles
genetic algorithms
Monte Carlo search
simulation
symbolic search
GNN over knowledge/evidence fields
multi-agent debate
counterfactual exploration
analogy search
```

Результат L3:

```text
Best Current Hypothesis Artifact
Negative Result Artifact
Open Research Artifact
```

---

# 11. Cognitive Evolution

Cognitive Evolution — довготривалий фоновий процес, який покращує майбутні Resolution.

Він не шукає відповідь на конкретну задачу.

Він шукає:

```text
нові закономірності;
нові евристики;
нові маршрути;
нові пояснення;
нові моделі;
нові способи перевірки;
нові способи зменшити майбутню вартість.
```

Це формалізує попередню ідею “цифрового аналога сновидінь” як **найвищу ескалацію когнітивних алгоритмів у полі парадоксальних рішень**.

---

# 12. Cognitive Field

Cognitive Field — дослідницька модель багатовимірного поля знань, Evidence, гіпотез, контекстів і зв’язків.

Воно **не є** Core Knowledge Graph.

Cognitive Field може містити:

```text
facts
hypotheses
contradictions
analogies
semantic links
domain bridges
failed experiments
contextual clusters
latent patterns
```

Мета Cognitive Field:

```text
підтримати Discover / Experiment / Validate;
знаходити міждисциплінарні зв’язки;
породжувати нові гіпотези;
зменшувати майбутню information cost.
```

---

# 13. “Glass Bead Game” Mode

Book V допускає окремий Research Mode:

```text
AIRA-Research-GBG
```

Мета:

```text
комбінувати віддалені області знань;
шукати структурні ізоморфізми;
виявляти приховані аналогії;
формувати нові концептуальні композиції;
генерувати перевірювані гіпотези.
```

Цей режим **MUST NOT** створювати Operational Artifact напряму.

Його результат:

```text
Hypothesis Artifact
Research Artifact
Open Research Artifact
```

---

# 14. Distributed Scientific Method

Distributed Scientific Method (DSM) — дослідницький механізм еволюції знань через спостереження, гіпотези, експерименти, реплікацію й валідацію. Цей напрям уже був виокремлений у попередньому Research Track як спосіб еволюції знань за моделлю, близькою до наукового методу. 

## 14.1 DSM lifecycle

```text
Observation
↓
Hypothesis
↓
Experiment
↓
Replication
↓
Peer Review
↓
Epistemic Update
↓
Artifact Promotion Candidate
```

## 14.2 DSM rules

DSM **MUST**:

```text
зберігати негативні результати;
підтримувати реплікацію;
відокремлювати Evidence від Interpretation;
фіксувати Counter Evidence;
не видаляти старі знання;
не використовувати TTL для Knowledge.
```

---

# 15. Capability Algebra

Capability Algebra залишається Research-напрямом.

Мета:

```text
формально описати композицію Capability;
визначити еквівалентність Capability;
порівнювати різнорідні виконавці;
будувати Capability substitutions;
оцінювати capability distance.
```

Приклади питань:

```text
Чи еквівалентні GPU inference і small-model ensemble?
Чи може human expert замінити model CSU?
Чи може symbolic solver замінити LLM?
Яка мінімальна Capability-композиція достатня для Result?
```

Capability Algebra не входить до Core.

Вона може стати Optimization / Research CSU.

---

# 16. Self-Organizing Constellations

Self-Organizing Constellations — Research-напрям, який досліджує локальне формування Execution Constellation без централізованого плану. Попередні документи вже відокремлювали ідею самоорганізації Constellation від детермінованого створення плану. 

## 16.1 Research goals

```text
local negotiation;
local repair;
hierarchical constellation formation;
constraint propagation;
failure-tolerant re-binding;
low-overhead distributed execution composition.
```

## 16.2 Constraints

SOEC **MUST NOT**:

```text
обходити Policy;
приховувати topology risks;
змінювати Execution Capsule semantics;
створювати неаудитований route;
залежати від глобального стану.
```

---

# 17. Advanced PHM

Advanced PHM досліджує:

```text
hardware degradation prediction;
behavioral reliability;
failure clustering;
contextual risk;
long-term resource aging;
thermal / power / memory instability;
execution interruption attribution.
```

PHM не приймає маршрутизаційні рішення.

PHM створює:

```text
Health Evidence Artifact
Risk Assessment Artifact
Capability Risk Metadata
```

Попередні документи вже розділили поведінкову ненадійність і технічну деградацію, що є критичним для справедливої оцінки вузлів.  

---

# 18. Knowledge / Artifact Evolution

Research Track досліджує довготривалу еволюцію Artifact.

Вхід:

```text
Evidence
Failures
Historical Events
Contradictions
External Research
User Feedback
Replication Results
```

Вихід:

```text
Knowledge Artifact
Hypothesis Artifact
Routing Heuristic Artifact
Verification Rule Artifact
Model Profile Artifact
Context Pattern Artifact
Negative Result Artifact
```

Artifact Lifecycle уже був виділений як універсальний життєвий цикл для всього, що створює AIRA. 

---

# 19. Goal Compiler

Goal Compiler залишається Research/User Plane-напрямом.

Book 0–III починаються з:

```text
Problem Statement
```

Goal може існувати вище ядра.

```text
Goal
↓
Problem Graph
↓
Problem Statement
↓
AIRA Operational Pipeline
```

Goal Compiler досліджує:

```text
розбиття цілі на Problem Graph;
довгострокові task chains;
project-level reasoning;
dependency management;
human-in-the-loop planning.
```

Goal Compiler **MUST NOT** бути вимогою Reference.

---

# 20. Universal Intent Theory

Universal Intent Theory лишається Research.

Причина:

```text
Core починається з Problem Statement;
Intent/Goal можуть бути User Plane або Research Plane;
Reference не повинен залежати від універсальної семантики наміру.
```

UIT може бути корисним для:

```text
Goal Compiler;
Context CSU;
Human Interaction CSU;
long-running planning;
multi-domain workflows.
```

---

# 21. Human Final Collapse Research

Book 0 фіксує Human Final Collapse як принцип.

Book V досліджує реалізації:

```text
alternative presentation;
risk visualization;
consequence modeling;
preference elicitation;
normative conflict detection;
deliberation support;
deferral mechanisms.
```

Research у цій області **MUST NOT** приховано обирати між епістемічно рівноправними нормативними альтернативами.

---

# 22. Collective Wisdom / Governance Research

Collective Wisdom не є Core.

Research-напрями:

```text
federated peer review;
distributed governance;
reputation without centralization;
anti-gaming contribution systems;
pluralistic policy evaluation;
community-curated artifacts;
trust domain evolution.
```

Ці механізми можуть покращувати мережу, але не повинні бути умовою сумісності.

---

# 23. Research CSU

Research CSU — спеціальний CSU-клас для експериментів.

## 23.1 Inputs

```text
Open Research Artifact
Hypothesis Artifact
Evidence Artifact
Research Policy Artifact
Budget Artifact
Historical Event Artifact
```

## 23.2 Outputs

```text
Research Artifact
Best Current Hypothesis Artifact
Negative Result Artifact
Validated Candidate Artifact
Artifact Promotion Candidate
```

## 23.3 Requirements

Research CSU **MUST**:

```text
бути ізольованим;
працювати через ABI;
публікувати Events;
публікувати Artifacts;
мати policy-controlled budget;
не впливати напряму на Operational Plane;
зберігати failure evidence;
декларувати scope;
підтримувати rollback / quarantine.
```

---

# 24. Research Safety Model

Research може бути дорогим, тривалим і стохастичним.

Тому він **MUST** мати окремі обмеження:

```text
budget limits;
policy limits;
privacy limits;
artifact quarantine;
human review for high-impact domains;
no direct operational promotion;
replication requirement for promotion;
negative-result preservation.
```

Research у high-stakes domains **MUST** проходити Human Review перед promotion.

---

# 25. Research Privacy

Research Plane **MUST NOT** використовувати приватні дані без explicit policy.

Заборонено:

```text
raw user prompt mining without consent;
private artifact reuse without policy;
cross-federation leakage;
training on protected data without declared scope;
de-anonymization research without explicit governance.
```

---

# 26. Research Failure Semantics

Research failure є цінним результатом.

Кожен failure **SHOULD** створювати:

```text
Negative Result Artifact
Failure Evidence Artifact
Open Research Artifact
```

Це запобігає повторенню марних експериментів і знижує future cost.

---

# 27. Research Conformance

Book V не має conformance-рівнів для AIRA-сумісності.

Але Research CSU може мати research-profile conformance:

```text
RV0 — isolated experiment
RV1 — artifact-producing experiment
RV2 — evidence-preserving experiment
RV3 — replicated experiment
RV4 — promotion-candidate experiment
```

---

# 28. Research-to-Reference RFC

Будь-який перехід з Research до Reference оформлюється RFC.

RFC **MUST** містити:

```text
problem statement;
research artifact refs;
evidence refs;
counter-evidence refs;
measured benefit;
compatibility analysis;
security analysis;
privacy analysis;
failure model;
rollback strategy;
conformance tests;
impact on Book 0–III.
```

Якщо RFC потребує зміни Core Ontology, він вважається **architecture-level proposal**, а не звичайним розширенням.

---

# 29. Explicitly Research-Only Items

Book V v0.1 фіксує такі напрями як Research-only:

```text
AIRA-RES-DSM   Distributed Scientific Method
AIRA-RES-SOEC  Self-Organizing Execution Constellations
AIRA-RES-CAP   Capability Algebra
AIRA-RES-PHM   Advanced PHM
AIRA-RES-KF    Knowledge / Cognitive Field
AIRA-RES-GBG   Glass Bead Game Mode
AIRA-RES-GC    Goal Compiler
AIRA-RES-UIT   Universal Intent Theory
AIRA-RES-CW    Collective Wisdom
AIRA-RES-HIE   Hypergraph / Higher-Order Intelligence Engines
AIRA-RES-EXP   Long-running Background Experiments
AIRA-RES-HFC   Human Final Collapse UX / Governance Research
```

---

# 30. Book V Relationship to Book IV

Book IV may include experimental toggles.

But any Book V feature in Book IV **MUST** be:

```text
disabled by default;
marked experimental;
isolated from Operational Plane;
excluded from Reference compatibility;
covered by safety policy;
excluded from core conformance tests.
```

---

# 31. Research Success Criteria

Book V is successful if it:

```text
generates reusable Evidence;
reduces future Resolution cost;
discovers new valid hypotheses;
improves verification strategies;
improves artifact quality;
improves capability matching;
improves reliability prediction;
produces negative results that prevent wasted work;
promotes only validated artifacts.
```

Research is unsuccessful if it:

```text
increases operational risk;
creates hidden dependencies;
requires Core changes without proof;
bypasses policy;
pollutes Operational Artifact pool;
creates unverifiable claims;
consumes unbounded resources without budget.
```

---

# 32. Research Track Formula

```text
Research =
    Evidence
  + Hypothesis
  + Experiment
  + Validation
  + Artifact Promotion
  - Direct Operational Authority
```

---

# 33. Status

Book V v0.1 формалізує Research Track AIRA.

Він закріплює:

```text
research isolation;
artifact-based experimentation;
distributed scientific method;
exploratory resolution;
cognitive evolution;
knowledge/cognitive field;
research CSU;
negative-result preservation;
promotion pipeline;
no direct operational influence.
```

Book V завершує набір базових документів AIRA:

```text
Book 0 — Foundations
Book I — Core Architecture & ABI
Book II — Protocol Specifications
Book III — CSU & ABI Contracts
Book IV — Reference Implementation
Book V — Research Track
```

Подальша робота має перейти до:

```text
1. консолідації Book 0–V в один чистий корпус;
2. видалення застарілих джерел з активного набору;
3. RFC для окремих протоколів;
4. формальних schemas;
5. conformance tests;
6. MVP implementation plan.
```
Наступний нормативний документ:

AIRA MVP Implementation Roadmap v0.1
