# Book 0 — Mathematical & Conceptual Foundations (аналіз)

**Джерело (immutable):** `Manifesto etc/Book 0 — Mathematical & Conceptual Foundations v0.1.md`  
**Тип джерела:** Normative, Draft v0.1, Depends on: none  
**Аналітик цикл:** Analyze-1  
**Confidence:** High (повне читання файлу)

---

## 1. Роль у корпусі

Book 0 — **конституція** AIRA: місія, first principles, онтологія, пайплайни, cost/escalation, межа ядра, freeze.  
Усі Book I–V мають бути сумісні з Book 0. Зміна = RFC-A (майже ніколи).

---

## 2. Місія (Evidence §1)

AIRA = мікроядерна федеративна система координованого зменшення невизначеності:  
`Problem Statement → Verified Result Artifact` з мінімальною сукупною вартістю через reuse і мінімально достатню композицію можливостей.

**Заборонені тлумачення:** GPU marketplace, LLM runtime, cloud inference, blockchain compute, centralized AI service.

**Inference:** Це свідомий pivot від ранньої P2P LLM/GPU моделі (`Meditation_About/1`).

---

## 3. First Principles A0–A7 (Evidence §2)

| ID | Принцип | Суть |
|----|---------|------|
| A0 | Reality Primacy | Не створює істину; збирає Evidence |
| A1 | Problem Primacy | Старт лише з Problem Statement |
| A2 | Context Primacy | Без Context — немає права інтерпретувати |
| A3 | Reuse Before Compute | Готове → знання → дешеве → важке → research |
| A4 | Minimal Necessary Action | Ескалація після провалу дешевшого контракту |
| A5 | Evidence Primacy | Evidence → Epistemic Processing → Knowledge |
| A6 | Human Final Collapse | Рівноправні альтернативи — вибір людини |
| A7 | Evolution Separation | Evolution не пише в Operational напряму |

---

## 4. Онтологія (Evidence §3)

**Core:** Problem Statement, Context, Evidence, Epistemic Status, Execution Intent, Execution Capsule, Capability, Artifact, Event, Policy, CSU, Verified Result Artifact.

**Не core:** GPU, LLM, Node, Driver, Scheduler, Blockchain, Wallet, HIE, Goal, Knowledge Graph, Execution Plan.

**Термінологічний зсув:** Driver→CSU, Job→Execution Capsule, Scheduler→Optimization CSU, Blockchain→Settlement/Audit CSU, Neurocoin→Contribution Credit.

---

## 5. Пайплайни (Evidence §4)

**Operational:** Interpret → Reduce → Differentiate → Organize → Materialize → Verify → VRA  

**Evolution:** Evidence → Aggregate → Discover → Validate → Promote → Operational Artifact  

`Differentiate` — для етичних/нормативних/відкритих задач.

---

## 6. Артефакти, епістеміка, Solution Space, Cost, Escalation

- §5: завжди Artifact (включно Negative / Open Research).  
- §6: шкала Contradicted…Axiom; Claim = Evidence+Counter+Status+Confidence+Scope+Fitness+History.  
- §7: Solution Space / Solution Uncertainty — механізм, не місія.  
- §8: Total Cost; CSU KPI = ΔSolution Uncertainty / Total Cost.  
- §9: L0 Ready … L4 Cognitive Evolution.

---

## 7. Межа ядра і freeze (Evidence §11–12)

**Kernel:** Object Model, ABI, Event Runtime, Policy Gate, Invariant Checker, Security Boundary.  
**Не в kernel:** AI/ML/HIE/Scheduler/Blockchain/GPU/LLM/optimization/knowledge algorithms.  
**Freeze:** нові сутності не в Core; лише CSU/Protocol/Algorithm/Artifact; ідеї → Research Track.

---

## 8. Звʼязок з Meditation (Inference + матриці)

| Шар | Внесок у Book 0 |
|-----|-----------------|
| MIND sTOR | A0/A2/A5/A7, Policy, Verify, Promote, microkernel, freeze→standard |
| AIRA Meditation 42–62 | Microkernel, SST (§7), GOA/Problem primacy, Lightweight Doctrine |
| AIRA Meditation 1–16 | Historical P2P LLM — superseded за SCD |

Деталі: `matrices/stor-to-book0.md`, `matrices/aira-meditation-to-book0.md`.

---

## 9. Відкриті питання для Analyze-2+

1. Чи повністю Book I реалізує §11 без витоку AI в kernel?  
2. Чи Conformance B0-* покриває Differentiate і Human Final Collapse?  
3. ~~Чи Goal (doc 62) конфліктує з §3.2?~~ — **ні** (Evidence фіналу 62: Goal поза мікроядром). Стежити лише за неповними цитатами GOA.
