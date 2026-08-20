# Phase D — Model Artifact & Inventory Plan v0.1

**Статус:** складено 2026-08-20; виконання стартоване. `#53`–`#56` DONE @ 9d5c311; перший OPEN — [`QUEUE.md`](../QUEUE.md) **#57**.  
**Джерела:** рішення розробника щодо інтерпретації [`EVO-3.md`](../EVO-3.md); Book 0–IV; Schema Pack; RFC Process; [`docs/implementation-status.md`](implementation-status.md).  
**Не канон backlog:** цей файл — загальний план і атомізація. Канон виконання — `QUEUE.md`.

Номери `#48`–`#55` у `EVO-3.md` §11 — **не** черга. Вони колізія з Phase C. Живі номери — `#53`–`#60` у `QUEUE.md`.

## 0. Порядок роботи

```text
загальний план (цей документ)
  → атоми без перекриття (§4)
    → лінійний хвіст QUEUE `#53`–`#60`
      → виконання: один OPEN рядок = один Analyze-цикл
```

Заборонено додавати поодинокі пункти в `QUEUE.md` поза цим планом. Download / sharing / rating / upgrade — не перший хвіст.

---

## 1. Класифікація

`EVO-3.md` описує **модельний контур AIRA**. Це valid architectural proposal, не Phase C і не негайний backlog.

```text
EVO-3 = valid architectural proposal
Status = not Phase C, not immediate queue
Classification = Phase D candidate + RFC-S / RFC-D / RFC-E
Core impact = none
Reference impact = only after this plan is copied into QUEUE
Research impact = rating / recommendation / upgrade (після D3)
```

**Правильна класифікація**

```text
Model layer =
    Artifact schemas
  + Capability declarations
  + Policy-controlled acquisition
  + CSU-based inventory / acquisition / sharing
  + Evidence-based rating
  + optional recommendation artifacts
```

**Неправильна класифікація**

```text
Model layer =
    Core entity
  + mandatory Model Registry
  + automatic download
  + global model leaderboard
  + required LLM runtime
  + Reference MVP / C1 dependency
```

Моделі — не початкова точка. Початкова точка лишається `Problem Statement`.

Канонічне представлення (не Core Ontology):

```text
model weights      → Artifact (CustomArtifact + payload schema)
model metadata     → Artifact
model ability      → Capability
model availability → Inventory Artifact
model trust        → Evidence / Epistemic Status
model usage        → Event / Evidence
model download     → Policy-gated CSU action (не в першому хвості)
```

CSU цього контуру (перший хвіст реалізує лише inventory; решта — пізніше):

```text
Local Model Inventory CSU     — D1
Model Acquisition CSU         — після D3 (D4)
Model Sharing CSU             — після D3 (D5)
Recommendation CSU            — RFC-R / пізніше (D7)
```

---

## 2. Інваріанти

1. `Model` / `LLM` / `GPU` / `Scheduler` не входять у Core Ontology і не в `aira-core`.
2. Inventory **не** є обов’язковим кроком C1 і **не** змінює Book 0 pipeline.
3. `OperationalPlane` drain/loop і C1 `Calculate 2 + 2` не змінюються цим етапом.
4. Download заборонений за замовчуванням (`auto_download = false`).
5. Sharing лише explicit opt-in (не в D0–D3).
6. Rating не є глобальним scalar score (не в D0–D3).
7. Upgrade recommendation — evidence-backed advisory, не marketplace і не «купити GPU» (не в D0–D3).
8. Inventory CSU має **scoped filesystem permission**; це не sandbox-клас `context-basic` / `reduction-basic`.
9. Файли моделей — immutable artifacts або зовнішні payload refs із hash verification.
10. Activation окрема від download (download не в D0–D3).
11. Capability = «можу виконати/надати модель X», не «node has model X».
12. Не редагувати `Manifesto etc/`, `Meditation_About/`. Зміна Book 0 pipeline = RFC-A (default Reject) — **поза цим планом**.

### Pipeline (Rank 4)

Inventory **не** новий канонічний крок. Це optional вхід для Reduce/Organize, якщо задача справді потребує модельного виконання:

```text
Reduce CSU
  may query Local Model Inventory Artifact
  may query Model Compatibility Evidence
  may produce Execution Candidate / Acquisition Candidate / Negative Result
```

Заборонено:

```text
Problem → Context → Model Inventory → Execution
```

У D0–D3 Reduce **ще не** підключається до inventory. Лише контракти, scan/list, compatibility evidence, policy default-deny.

---

## 3. Scope першого хвоста vs пізніше

**У цьому плані (D0–D3)**

```text
payload schemas (CustomArtifact, без нового canonical ArtifactType);
local scan / list / compatible (read-only CLI);
hardware/backend compatibility → Evidence;
policy default-deny download;
conformance/schema fixtures для нових payload;
RFC-S (+ RFC-D/RFC-E перед відповідним кодом).
```

**Свідомо не в першому хвості**

```text
model download / activation (D4)
custom model publish / sharing (D5)
contextual rating / popularity (D6) — RFC-R
upgrade recommendations (D7) — RFC-R
новий canonical enum ModelArtifact
зміна Book 0 pipeline / Core / C1 CI semantics
мережевий discovery моделей / federation model market
```

**Критерій завершення D0–D3** (коли хвіст потрапить у QUEUE і всі рядки DONE)

```text
є payload schemas + fixtures;
aira models scan|list|compatible — локально, без мережі, без download;
кожна класифікація runnable/incompatible/unknown має reason + Evidence Artifact;
будь-яке завантаження без policy = DENY + Event/Evidence; implicit download немає;
C1 pipeline і aira-core не змінені;
docs/implementation-status.md оновлено: model layer = PARTIAL (schemas+inventory), HTTP/peer лишаються POST-MVP.
```

---

## 4. Атоми першого хвоста

Нумерація `#53+` у [`QUEUE.md`](../QUEUE.md). Analyze-номери лінійно після Analyze-87.

Один payload schema file = один рядок (як Phase C: один файл-контракт).

| ID | Підфаза | Атом | Done when | Не в цьому рядку |
|----|---------|------|-----------|------------------|
| `#53` | D0.1 | Payload schema `ModelArtifact` (`CustomArtifact` + `$id` `aira:schema:model:artifact:0.1`) | schema + valid/invalid fixtures; `schema validate --fixtures` зелений | Profile/Inventory; canonical `ArtifactType` enum; downloader |
| `#54` | D0.2 | Payload schema `ModelProfileArtifact` | schema + fixtures | Inventory; hardware scan |
| `#55` | D0.3 | Payload schema `LocalModelInventoryArtifact` | schema + fixtures | CLI scan; мережа |
| `#56` | D0.4 | Payload schema `ModelCompatibilityEvidence` | schema + fixtures; поля reason / confidence / scope | auto-download; rating score |
| `#57` | D0.5 | Payload schema `ModelAcquisitionPolicyArtifact` | schema + fixtures; default `auto_download=false` | сам downloader; allowlist runtime |
| `#58` | D1 | Local inventory, read-only: `aira models scan\|list`; Inventory CSU зі scoped FS | scan локальних каталогів; immutable inventory artifact; Event `CustomEvent` або вузький тип **без** зміни C1; без мережі; без download | compatible resolver; Policy runtime download |
| `#59` | D2 | Compatibility resolver: hardware/backend/profile → runnable / incompatible / unknown | кожна модель має reason; результат = Evidence Artifact; без download | acquisition CSU; CLI download |
| `#60` | D3 | Acquisition policy runtime: default DENY download | завантаження без policy = DENY; PolicyDenied → Event/Evidence; implicit download відсутній | реальний download (D4); sharing |

Перед `#53`: RFC-S (payload schemas, не нові Core entities, не RFC-A). Перед `#58`: RFC-D (Inventory CSU) + RFC-E (CLI). RFC пишуться в тому ж Analyze-циклі, що й перший код/схема відповідного рядка, або окремим docs-рядком, якщо розробник відкриє його в QUEUE — **не** змішувати RFC-S із downloader.

Hardware profile для D2 — локальний дескриптор/payload, не Core entity і не обов’язковий D0.

---

## 5. Лінійний порядок (резерв)

```text
#53 ModelArtifact payload schema
  → #54 ModelProfile payload schema
    → #55 LocalModelInventory payload schema
      → #56 CompatibilityEvidence schema
        → #57 AcquisitionPolicy schema
          → #58 local scan/list (Inventory CSU, scoped FS)
            → #59 compatibility resolver
              → #60 policy default-deny (no download)
```

Паралелі немає. Не стартувати `#N+1`, доки `#N` не DONE в `QUEUE.md`.

**Чому так:** контракти раніше коду; inventory раніше compatibility; policy DENY раніше будь-якого acquisition CSU.

---

## 6. Відкладені підфази (не нумерувати в QUEUE зараз)

| ID | Зміст | RFC | Умова старту |
|----|--------|-----|----------------|
| D4 | Model download + hash/signature verify + activation окремо | RFC-D + RFC-E | D3 DONE |
| D5 | Custom model publish / share, opt-in | RFC-D | D3 DONE; не вимагає D4, але не в першому хвості |
| D6 | Contextual model rating evidence (не global score) | RFC-R | після D3; не C1 |
| D7 | Upgrade recommendation artifact (advisory) | RFC-R | після D3; не marketplace |

Новий загальний план або addendum до цього файлу — перед копіюванням D4–D7 у QUEUE.

---

## 7. RFC-карта

| Клас | Навіщо | Коли |
|------|--------|------|
| RFC-S | payload schemas; **не** піднімати `ModelArtifact` у canonical enum на старті | до/разом із `#53` |
| RFC-D | Inventory CSU contract; sandbox ≠ basic CSU | до/разом із `#58` |
| RFC-E | `aira models scan\|list\|compatible`; local cache scan | до/разом із `#58`–`#59` |
| RFC-T | fixtures/conformance для нових payload | разом зі схемами |
| RFC-A / RFC-B | Book 0 pipeline / Core | **заборонені** цим планом |
| RFC-R / RFC-P | rating, recommendation, popularity, placement | лише D6–D7 |

Default: RFC-A Reject, якщо хтось запропонує крок `Model Inventory` у канонічному pipeline.

---

## 8. Фільтр задачі

1. Чи це контракт / локальний read-only inventory / compatibility / default-deny — без download?  
2. Чи Core і C1 pipeline не змінюються?  
3. Чи `ArtifactType` лишається `CustomArtifact` + payload `$id`?  
4. Чи є один вимірюваний Done when і колонка Out?  
5. Чи номер не `#48`–`#52` і не EVO-3 §11?

Ні → не в перший хвіст Phase D.
