# Phase D — Model Artifact & Inventory Plan v0.1

**Статус:** складено 2026-08-20; перша хвиля D0–D3 (`#53`–`#60`) **DONE** @ 21d90a5. **Addendum D4** (`#61`–`#64`) **DONE** @ ffcf66f. **Addendum D5** відкрито рішенням розробника 2026-08-20 → QUEUE `#65`–`#68`. D6–D7 **не** в QUEUE.  
**Джерела:** рішення розробника щодо інтерпретації [`EVO-3.md`](../EVO-3.md); Book 0–IV; Schema Pack; RFC Process; [`docs/implementation-status.md`](implementation-status.md).  
**Не канон backlog:** цей файл — загальний план і атомізація. Канон виконання — `QUEUE.md`.

Номери `#48`–`#55` у `EVO-3.md` §11 — **не** черга. Вони колізія з Phase C. Живі номери — `#53`+ у `QUEUE.md`.

## 0. Порядок роботи

```text
загальний план (цей документ)
  → атоми без перекриття (§4 / Addendum D4 / Addendum D5)
    → лінійний хвіст QUEUE
      → виконання: один OPEN рядок = один Analyze-цикл
```

Заборонено додавати поодинокі пункти в `QUEUE.md` поза цим планом / addendum. D6–D7 лишаються відкладеними.

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

## 6. Відкладені підфази

| ID | Зміст | RFC | Умова старту | Статус |
|----|--------|-----|----------------|--------|
| D4 | Model download + hash/signature verify + activation окремо | RFC-D + RFC-E | D3 DONE | **DONE** — Addendum D4 / QUEUE `#61`–`#64` @ ffcf66f |
| D5 | Custom model publish / share, opt-in | RFC-D + RFC-E | D3 DONE; не вимагає D4 | **DONE** — Addendum D5 / QUEUE `#65`–`#68` @ 7719e92 |
| D6 | Contextual model rating evidence (не global score) | RFC-R | після D3; не C1 | **не в QUEUE** |
| D7 | Upgrade recommendation artifact (advisory) | RFC-R | після D3; не marketplace | **не в QUEUE** |

Новий загальний план або ще один addendum — перед копіюванням D6–D7 у QUEUE.

---

## 6a. Addendum D4 (2026-08-20) — лише download / verify / activate

**Рішення розробника:** відкрити **лише D4**. D5–D7 не копіювати в QUEUE.

**Інваріанти D4 (додатково до §2)**

1. Policy gate з D3 лишається обов’язковим: без policy / `auto_download=false` → DENY (без змін семантики `#60`).
2. **Activation окрема від download** (Book / EVO-3 §8.3–8.4): download не активує і не виконує модель.
3. Verify hash/signature **перед** activation; quarantine для невдалих.
4. Envelope лишається `CustomArtifact` + payload `$id`; canonical `ArtifactType::ModelArtifact` — **Out**.
5. C1 / `aira-core` / Book 0 pipeline — **не змінювати**.
6. Sharing / rating / recommendation — **Out** (D5–D7).
7. **Перший D4-хвіст: локальне джерело** (`--source` file path → quarantine під scoped `<root>/models`). Remote HTTP/URL fetch — **Out** цього addendum (окремий майбутній addendum або рядок, якщо знадобиться).

**Атоми → QUEUE `#61`–`#64`** (Analyze-96+)

| ID | Підфаза | Атом | Done when | Не в цьому рядку |
|----|---------|------|-----------|------------------|
| `#61` | D4.1 | Policy gate **ALLOW** path | при `auto_download=true` + наявній policy → decision **ALLOW** (+ Event); DENY-шляхи `#60` не зламані; **без** byte transfer | quarantine fetch; verify; activate; HTTP |
| `#62` | D4.2 | Fetch у quarantine (local `--source` only) | після ALLOW: копія ваг у `<root>/models/quarantine/…`; Event; **без** activation | hash verify promote; activate; remote URL |
| `#63` | D4.3 | Verify content_hash + signature | mismatch/unsigned → reject + Evidence, лишається quarantine; match → staging `verified/` | activate; inventory promote; HTTP |
| `#64` | D4.4 | Activate окремо | explicit activate: verified → cache; inventory update; `ModelInstalled`-style CustomEvent; **без** auto-execution | sharing; rating; remote registry |

```text
#61 ALLOW gate
  → #62 quarantine fetch (local source)
    → #63 verify hash/signature
      → #64 activate (explicit)
```

**RFC:** RFC-D (Acquisition CSU download/activate contract) + RFC-E (`aira models download|activate` поведінка) — у циклах `#61`/`#62` відповідно, не змішувати з D5.

---

## 6b. Addendum D5 (2026-08-20) — лише custom publish / share (opt-in)

**Рішення розробника:** відкрити **лише D5**. D6–D7 не копіювати в QUEUE.

**Інваріанти D5 (додатково до §2)**

1. `share_custom_models = false` за замовчуванням; publish/share без explicit opt-in → DENY + Evidence.
2. Користувач публікує **signed Artifact descriptor**, не «голий файл» і **не** форсує global availability (EVO-3 §8 / §11 `#53`).
3. Envelope = `CustomArtifact` + payload `$id`; canonical `ArtifactType::ModelArtifact` — **Out**.
4. Sharing CSU (або окремий crate) — `network=none` у першому хвості; **немає** remote advertise / marketplace / DHT model registry.
5. CSU↛CSU firewall: Sharing не залежить від Inventory/Acquisition Cargo-dep; CLI оркеструє.
6. C1 / `aira-core` / Book 0 — **не змінювати**.
7. Rating / recommendation / settlement — **Out** (D6–D7).
8. Download/activate чужих shared моделей через мережу — **Out** цього addendum (D4 local path уже є; remote share fetch — окремий майбутній addendum).

**Атоми → QUEUE `#65`–`#68`** (Analyze-100+)

| ID | Підфаза | Атом | Done when | Не в цьому рядку |
|----|---------|------|-----------|------------------|
| `#65` | D5.1 | Payload schema `ModelShareOffer` | `aira:schema:model:share-offer:0.1` + fixtures; envelope CustomArtifact | policy runtime; publish CLI; network |
| `#66` | D5.2 | Share policy gate (`share_custom_models`) | absent/false → DENY publish + Event/Evidence; true → ALLOW decision **без** publish bytes | actual publish; capability advertise; rating |
| `#67` | D5.3 | Local publish signed descriptor | після ALLOW: з `models/cache` → signed ModelArtifact (+ ShareOffer link); Event; **без** remote push | capability advertise; remote registry; rating |
| `#68` | D5.4 | Local capability advertisement | CustomArtifact capability ad з publish; CLI `models publish\|share`; visibility local-only | federation push; DHT; rating (D6); marketplace |

```text
#65 ShareOffer schema
  → #66 share_custom_models gate
    → #67 local publish signed descriptor
      → #68 local capability advertisement
```

**RFC:** RFC-S (ShareOffer schema) → RFC-D (Sharing CSU) + RFC-E (`aira models publish|share`) у `#66`–`#68`; не змішувати з D6 rating.

---

## 7. RFC-карта

| Клас | Навіщо | Коли |
|------|--------|------|
| RFC-S | payload schemas; **не** піднімати `ModelArtifact` у canonical enum на старті | до/разом із `#53` |
| RFC-D | Inventory CSU contract; sandbox ≠ basic CSU | до/разом із `#58` |
| RFC-E | `aira models publish` share gate exits | `#66` / RFC-0015 |
| RFC-D | share_custom_models publish gate | `#66` |
| RFC-D/E | local publish signed ModelArtifact + ShareOffer | `#67` / RFC-0016 |
| RFC-D/E | local capability advertisement + CLI share | `#68` / RFC-0017 |
| RFC-S | ModelShareOffer payload schema | `#65` / RFC-0014 |
| RFC-E | `aira models activate` + inventory scan of cache | `#64` / RFC-0013 |
| RFC-D | Activate verified → cache (no execution) | `#64` |
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
