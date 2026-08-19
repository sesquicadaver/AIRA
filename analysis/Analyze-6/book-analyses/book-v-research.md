# Book V — Research Track (аналіз)

**Джерело:** `Manifesto etc/Book V — Research Track v0.1.md`  
**Type:** Research / Non-Normative  
**Depends on:** Book 0–IV  
**Формат:** structured digest §1–33 після повного читання (955 рядків)  
**Confidence:** High (Evidence); Medium (Inference)

---

## 1. Роль і межа

**Evidence (§0):** Type = Research / Non-Normative; exports Research Governance, Artifact Model, Research CSU Profiles, Exploratory Resolution, Cognitive Evolution, Promotion Pipeline, Safety Boundaries.  
**Evidence (§0):** Сумісність = Book 0–III; Book IV = reference; Book V = як research існує без руйнування Core/Protocols/CSU/Operational Plane.  
**Evidence (§1):** Purpose — зменшення невизначеності, гіпотези, алгоритми, майбутні Resolution, Knowledge/Evidence/Artifact ecosystem; **не** прямий вплив на Operational Plane.  
**Evidence (§2):** Non-goals — Core Object Model, Stable ABI, protocol compatibility, mandatory CSU, Reference behavior, required ML/optimizer/blockchain/global KG. Жоден компонент Book V не є умовою сумісності.

**Inference:** Book V = ізольований еволюційний контур; Standard formula лишається 0–III + Conformance (SCD).

---

## 2. Research Boundary і два площини

**Evidence (§3):** MUST NOT змінювати Core / обходити Stable ABI / Policy Gate / Artifact in-place / створювати Operational Artifact напряму / прихований normative collapse / вимагати змін 0–III без RFC / впливати на Operational без Artifact Promotion.  
**Evidence (§3):** MAY створювати Evidence / Hypothesis / Research / Best Current Hypothesis / Negative Result / Open Research / Promotion Candidate artifacts.  
**Evidence (§4.1–4.2):** Operational Plane = PS→Interpret→Reduce→Differentiate→Organize→Materialize→Verify→VRA; Research Plane = Evidence→Aggregate→Discover→Experiment→Validate→Promote→Operational Artifact Candidate. Research асинхронний; Operational **не чекає** Research.

**Inference:** Dual-plane — ключовий інваріант: research покращує майбутнє, не блокує поточне.

---

## 3. Admission, Artifact Model, Lifecycle, Promotion

**Evidence (§5):** Admission — не потрібне для мінімальної сумісної AIRA; може покращити якість/вартість/швидкість/надійність; недостатньо доказів для Reference; потребує експериментів; реалізоване як CSU/Algorithm/Artifact/Protocol Extension. Нова фундаментальна сутність → спочатку Research, не Reference.  
**Evidence (§6):** Research Artifact YAML MUST: artifact_id, type=ResearchArtifact, research_domain, hypothesis/evidence/counter_evidence refs, method_ref, experiment_refs, result_status, confidence, scope, risk_descriptor, policy_refs, provenance_refs, created_at, signature.  
**Evidence (§6.1):** result_status ∈ proposed|running|replicated|failed|contradicted|validated|promoted_candidate|deprecated.  
**Evidence (§7):** Lifecycle Proposal→…→Promotion Candidate→Reference RFC або Archive; жоден Research Artifact не для Operational напряму.  
**Evidence (§8–8.1):** Promotion тільки через pipeline (Evidence Review→…→Reference RFC→Optional/Reference/Rejected); criteria: measurable cost↓, Book 0–III compat, no Core change, conformance tests, failure semantics, rollback, no hidden normative collapse.

---

## 4. Budget і Escalation (L0–L4)

**Evidence (§9):** Окремий Research Budget (Discovery/Experiment/Validation/Promotion/Energy/Risk/Human Review); MUST NOT витісняти Operational; idle/voluntary/federations/background/scheduled/donated.  
**Evidence (§10):** Escalation L0 Ready Solution → L1 Verified Knowledge → L2 Deterministic Resolution → L3 Exploratory Resolution → L4 Cognitive Evolution.  
**Evidence (§10.1):** L3 — нові гіпотези / Solution Field / cross-domain; MAY ML/LLM/GA/MC/sim/symbolic/GNN/debate/counterfactual/analogy; outputs BCH / Negative / Open Research.  
**Evidence (§11):** Cognitive Evolution — фоновий процес (не відповідь на конкретну задачу); формалізує «цифровий аналог сновидінь» як найвищу ескалацію в полі парадоксальних рішень.

**Tension:** L0–L4 тут — **escalation of resolution methods**, не Book I conformance L0–L3 — TF6-002.

---

## 5. Cognitive Field, GBG, DSM

**Evidence (§12):** Cognitive Field ≠ Core Knowledge Graph; facts/hypotheses/contradictions/analogies/…; підтримка Discover/Experiment/Validate.  
**Evidence (§13):** Mode `AIRA-Research-GBG` — ізоморфізми/аналогії/композиції; MUST NOT Operational Artifact; outputs Hypothesis/Research/Open Research.  
**Evidence (§14):** DSM — Observation→…→Epistemic Update→Promotion Candidate; MUST зберігати негативні результати, реплікацію, Evidence≠Interpretation, Counter Evidence, не видаляти знання, не TTL для Knowledge.

---

## 6. Capability Algebra, SOEC, Advanced PHM, Artifact Evolution

**Evidence (§15):** Capability Algebra — Research; композиція/еквівалентність/substitutions/distance; не Core; може стати Optimization/Research CSU.  
**Evidence (§16):** SOEC — local negotiation/repair/hierarchical formation; MUST NOT обходити Policy / ховати topology risks / міняти Capsule semantics / unaudited route / global state dependency.  
**Evidence (§17):** Advanced PHM — degradation/reliability/clustering/aging; **не** маршрутизує; outputs Health Evidence / Risk Assessment / Capability Risk Metadata.  
**Evidence (§18):** Knowledge/Artifact Evolution — Evidence/Failures/… → Knowledge/Hypothesis/Routing Heuristic/Verification Rule/Model Profile/Context Pattern/Negative Result.

---

## 7. Goal Compiler, UIT, Human Final Collapse, Collective Wisdom

**Evidence (§19):** Goal Compiler = Research/User Plane; Core починається з Problem Statement; Goal→Problem Graph→PS→Operational; MUST NOT вимога Reference.  
**Evidence (§20):** UIT лишається Research; Intent/Goal = User/Research Plane; Reference не залежить від універсальної семантики наміру.  
**Evidence (§21):** Human Final Collapse — принцип у Book 0; Book V досліджує UX/governance реалізації; MUST NOT приховано обирати між епістемічно рівноправними нормативними альтернативами.  
**Evidence (§22):** Collective Wisdom — federated peer review, distributed governance, reputation, anti-gaming, pluralistic policy, community artifacts, trust domains; не умова сумісності.

**Inference:** Узгоджується з Meditation 62 (Goal поза мікроядром → Goal Compiler Driver) і Book 0 Goal ban у Core.

---

## 8. Research CSU, Safety, Privacy, Failure, Conformance

**Evidence (§23):** Research CSU inputs/outputs; MUST isolated, ABI, Events/Artifacts, policy budget, no direct Operational, failure evidence, scope, rollback/quarantine.  
**Evidence (§24–25):** Safety — budget/policy/privacy/quarantine/human review/no direct promotion/replication/negative-result; Privacy MUST NOT raw prompt mining / private reuse / cross-federation leakage / protected training / de-anon без governance.  
**Evidence (§26):** Failure → Negative Result / Failure Evidence / Open Research (SHOULD).  
**Evidence (§27):** Немає AIRA-compatibility levels; research-profile RV0–RV4 (isolated → promotion-candidate).

**Tension:** RV0–RV4 — ще одна шкала поруч із L/P/C/R/SCD — TF6-001.

---

## 9. RFC bridge, Research-only IDs, Book IV relation

**Evidence (§28):** Research→Reference лише через RFC з повним пакетом (problem, refs, benefit, compat/security/privacy, failure, rollback, conformance, impact on 0–III); Core Ontology change = architecture-level proposal.  
**Evidence (§29):** Explicit Research-only: AIRA-RES-DSM, SOEC, CAP, PHM, KF, GBG, GC, UIT, CW, HIE, EXP, HFC.  
**Evidence (§30):** Book IV MAY experimental toggles; MUST disabled by default, marked experimental, isolated, excluded from Reference compatibility & core conformance, safety policy.  
**Evidence (§31–32):** Success/unsuccess criteria; Formula: Research = Evidence+Hypothesis+Experiment+Validation+Promotion − Direct Operational Authority.  
**Evidence (§33):** Book V завершує базовий набір 0–V; далі — консолідація корпусу, prune Historical, RFC, schemas, conformance tests, MVP plan; наступний нормативний документ = MVP Implementation Roadmap.

**Inference:** Analyze-6 закриває книжковий ланцюг Manifesto Books; наступний Analyze-7+ логічно = SCD/Conformance/MVP synthesis, не нова «книга».
