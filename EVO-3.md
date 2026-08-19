**Статус (2026-08-20):** proposal джерело для Phase D. Канон виконання — [`QUEUE.md`](QUEUE.md) `#53`–`#60`. Модельний контур = Artifact + Capability + Policy + CSU, **не** Core. Номери `#48`–`#55` у §11 **не** є чергою (колізія з Phase C). План: [`docs/phase-d-plan.md`](docs/phase-d-plan.md).

Проблема: набір моделей у різних користувачів може і буде відрізнятися. 
Причини: різна потужність cpu, gpu, ram, ssd тощо. 
Гіпотеза: користувач повинен надавати дозвіл на завантаження моделей, які можуть працювати на його "залізі". 
Також має бути реалізована підсистема шерінгу власних моделей (коли користувач може поділитися власною кастомною llm. 
Але для цього потрібно вирішити декілька суто технічних задач: локальне визначення і підтримання актуального списку моделей для кожного клієнта. 
Також бажано створити можливість продивлятися рекомендації по апгрейду "заліза" і моделей. 
Для моделей також актуальний рейтинг (популярність, сфера використання тощо).

АНАЛІЗ
Це реальна архітектурна прогалина. Її треба закрити, але **не через додавання Model у Core**.

Канонічно AIRA вже зафіксована як система `Problem Statement → Verified Result Artifact` з мінімальною сукупною вартістю, а не як LLM runtime чи GPU marketplace.  Book I прямо забороняє Core містити LLM/GPU/model/scheduler/domain logic.  Отже моделі мають бути представлені як **Artifacts + Capabilities + Policy**, а вся логіка керування ними — як CSU.

# 1. Суть проблеми

У різних користувачів реально буде різний набір моделей через:

```text
VRAM
RAM
CPU features
GPU backend
SSD capacity
disk bandwidth
network bandwidth
OS / drivers
backend support
quantization support
local cache state
user policy
```

Тому не можна виходити з припущення:

```text
AIRA node has model X
```

Правильне припущення:

```text
AIRA node may expose capability to use / host / download / share model artifacts
under local hardware and policy constraints.
```

Ранні документи вже бачили Model Registry як каталог model/version/tokenizer/backend/quantization/hash/shard map, але в канонічній архітектурі це має бути не Core Registry, а Artifact/Capability/CSU-рівень. 

---

# 2. Гіпотеза про дозвіл користувача — правильна

Автоматичне завантаження моделей без явної політики не можна дозволяти.

Причини технічні:

```text
моделі великі;
SSD може бути обмеженим;
модель може не запускатися на локальному залізі;
download може зірвати поточну роботу;
модель може мати невідповідну ліцензію/політику;
кастомна модель може бути шкідливою або пошкодженою;
кеш моделей може витіснити потрібні артефакти;
користувач може не хотіти шарити конкретні моделі.
```

Тому має бути окрема локальна політика:

```text
Model Acquisition Policy
```

Але це **Policy Artifact**, не нова сутність ядра.

Приклад рішень:

```text
allow_auto_download = false by default
allow_download_if_size_below = 8GB
allow_quantized_only = true
allow_untrusted_models = false
max_model_cache_size = 200GB
share_custom_models = explicit opt-in
```

---

# 3. Мінімальна правильна архітектура

Не треба створювати великий “Model Management Plane”. Достатньо трьох CSU.

## 3.1 Local Model Inventory CSU

Відповідає за локальний список моделей.

Функції:

```text
сканувати локальний model cache;
визначати доступні backend;
визначати runnable models;
визначати incompatible models;
оновлювати Local Model Inventory Artifact;
емітити ModelInventoryUpdated Event;
не завантажувати моделі самостійно.
```

Вхід:

```text
Hardware Capability Artifact
Backend Capability Artifact
Local filesystem scan
Model Profile Artifacts
Policy Artifact
```

Вихід:

```text
Local Model Inventory Artifact
Runnable Model Set Artifact
ModelCompatibilityEvidence
```

## 3.2 Model Acquisition CSU

Відповідає за контрольоване завантаження.

Функції:

```text
перевірити політику користувача;
перевірити hardware compatibility;
перевірити storage budget;
перевірити hash/signature;
завантажити модель як immutable Artifact;
оновити inventory.
```

Важливо: він не вибирає “кращу модель”. Він лише матеріалізує дозволений Model Artifact локально.

## 3.3 Model Sharing CSU

Відповідає за публікацію власних моделей.

Функції:

```text
створити Model Artifact Descriptor;
порахувати content hash;
додати provenance;
додати model card / capabilities / constraints;
підписати artifact;
опублікувати capability advertisement;
дозволити або заборонити download іншими вузлами.
```

Раніше Model Distribution уже розділялась від Job execution: Registry → Node Cache → Verification → Activation. Це правильна ідея, але її треба перевести в Artifact Protocol/CSU, без прямої залежності від Job/Node/Scheduler. 

---

# 4. Як це вписується в AIRA

## Core

Без змін.

Core бачить лише:

```text
Artifact
Capability
Policy
Event
CSU
```

## Book II / Protocols

Потрібні не нові фундаментальні протоколи, а розширення існуючих:

```text
Artifact Protocol       — публікація/отримання model artifacts
Discovery Protocol      — пошук model capability
Capability Advertisement — “я можу виконати/надати модель X”
Identity Protocol       — підпис автора / вузла / федерації
```

Book II вже визначає, що Discovery шукає Capability/CSU, а не Node, і не зобов’язує до конкретного алгоритму. 

## Book III / CSU

Це звичайні CSU. Book III визначає CSU як ізольовані замінні модулі, що мають Identity, публікують Capability, працюють через ABI, Events, Artifacts і Policy. 

## Book IV / Reference

У Reference варто додати лише мінімум:

```text
aira models scan
aira models list
aira models compatible
aira models policy show/set
```

Автозавантаження і шаринг — не обов’язкові для локального MVP. Book IV прямо вимагає мінімальну реалізацію без model hosting system. 

---

# 5. Мінімальні артефакти

## 5.1 Model Artifact

Описує конкретні ваги або shard.

```yaml
artifact_type: ModelArtifact
model_id: string
format: gguf | safetensors | custom
quantization: string
parameter_class: string
content_hash: hash
tokenizer_ref: artifact_ref
license_policy_ref: policy_ref
provenance_refs: [artifact_ref]
signature: signature
```

## 5.2 Model Profile Artifact

Описує вимоги і поведінку моделі.

```yaml
artifact_type: ModelProfileArtifact
model_ref: artifact_ref
required_vram_gb: number
required_ram_gb: number
min_disk_gb: number
supported_backends: [string]
supported_quantizations: [string]
context_length: integer
modalities: [text | vision | audio | code | embedding]
domains: [string]
estimated_latency_class: string
evidence_refs: [evidence_ref]
```

## 5.3 Local Model Inventory Artifact

Описує локальний стан клієнта.

```yaml
artifact_type: LocalModelInventoryArtifact
host_ref: identity_ref
installed_models: [artifact_ref]
runnable_models: [artifact_ref]
downloadable_compatible_models: [artifact_ref]
incompatible_models:
  - model_ref: artifact_ref
    reason: string
cache_budget:
  total_gb: number
  used_gb: number
  reserved_gb: number
updated_at: timestamp
signature: signature
```

## 5.4 Model Rating Evidence Artifact

Рейтинг не має бути одним числом.

```yaml
artifact_type: ModelRatingEvidence
model_ref: artifact_ref
domain: string
task_class: string
hardware_class: string
usage_count: integer
success_rate: number
verification_score: number
latency_observed: object
cost_observed: object
user_feedback_score: number
evidence_refs: [evidence_ref]
scope: object
confidence: number
```

Це узгоджується з Evidence/Epistemic моделлю: знання не є TTL-кешем, а має Evidence, scope, confidence і статус.  

---

# 6. Рейтинг моделей

Рейтинг потрібен, але не як глобальний leaderboard.

Правильно:

```text
model_score(context, hardware, task, policy)
```

а не:

```text
global_model_score
```

Мінімальні координати рейтингу:

```text
популярність;
сфера використання;
якість верифікованих результатів;
швидкість на класах hardware;
вартість запуску;
стабільність backend;
розмір;
ліцензійна/політична сумісність;
частота оновлень;
довіра до publisher;
кількість успішних локальних запусків;
кількість failure evidence.
```

Популярність — лише одна вісь. Вона не повинна домінувати.

Приклад:

```text
Qwen-code model може мати високий рейтинг для coding,
низький для medical,
добрий для GPU 16GB INT4,
поганий для CPU-only.
```

---

# 7. Рекомендації по апгрейду заліза

Це корисно, але має бути **Recommendation CSU**, не Core.

Він повинен аналізувати не “що модно купити”, а bottleneck evidence.

Вхід:

```text
Local Hardware Capability Artifact
Local Model Inventory Artifact
Task History Evidence
Failed Compatibility Evidence
Observed Latency Evidence
Cache Pressure Evidence
User Policy
```

Вихід:

```text
Upgrade Recommendation Artifact
```

Формат рекомендації:

```yaml
recommendation_type: hardware | model | storage | backend | none
reason: string
evidence_refs: [evidence_ref]
expected_gain:
  runnable_model_count_delta: integer
  latency_delta_estimate: string
  local_execution_ratio_delta: string
  cache_pressure_delta: string
alternatives:
  - smaller_quantization
  - remote_execution
  - download_model
  - hardware_upgrade
confidence: number
```

Приклад правильної логіки:

```text
Якщо користувач часто просить coding/reasoning,
а локально проходять лише 7B INT4,
і 14B/32B не запускаються через VRAM,
система рекомендує:
1. спробувати кращу квантизацію;
2. завантажити меншу specialized coding model;
3. лише потім — апгрейд GPU/VRAM.
```

Це прямо відповідає принципу AIRA: мінімізувати сукупну вартість, а не автоматично купувати/качати більше. Book 0 фіксує Reuse Before Compute і Minimal Necessary Action. 

---

# 8. Шеринг власних моделей

Потрібен, але з обмеженнями.

## 8.1 Користувач публікує не “файл”, а Artifact

```text
custom model weights
+ tokenizer
+ config
+ model profile
+ provenance
+ signature
+ policy
+ optional rating evidence
```

## 8.2 Інші користувачі бачать Capability

Наприклад:

```text
publisher X exposes:
  coding.llm.decode
  gguf
  int4
  max_context=32k
  model_hash=...
```

## 8.3 Завантаження тільки після Policy Gate

```text
request model
↓
check trust
↓
check policy
↓
check hardware compatibility
↓
check disk budget
↓
download artifact
↓
verify hash/signature
↓
activate locally
```

## 8.4 Обов’язкові захисти

```text
content-addressed storage;
signature verification;
publisher identity;
model provenance;
license/policy metadata;
sandboxed loading;
hash verification before activation;
quarantine for untrusted models;
no auto-execution after download.
```

Artifact Lifecycle уже встановлює правильний принцип: усе, що створює AIRA, є immutable Artifact, а зміни відбуваються через нові версії, не через mutation. 

---

# 9. Локальне підтримання актуального списку моделей

Потрібен deterministic loop:

```text
on startup:
  scan local model cache
  scan backend support
  load hardware profile
  validate model descriptors
  build runnable set
  emit ModelInventoryUpdated

periodically:
  check artifact registry updates
  check model profile supersession
  check broken/missing files
  check cache pressure
  update inventory only if changed

on model download:
  verify hash/signature
  publish ModelInstalled event
  rebuild runnable set

on hardware/backend change:
  rebuild compatibility matrix
```

Не треба опитувати всю мережу. Достатньо локального inventory + cached federation metadata + explicit discovery when needed.

Це відповідає принципу Minimal Decision Information: система повинна використовувати мінімально достатню інформацію, а не бачити всю мережу. 

---

# 10. Як це впливає на Resolution Pipeline

Поточний pipeline має змінитися не структурно, а змістовно.

Було:

```text
Problem Statement
↓
Context
↓
Reuse / Reduction
↓
Execution
↓
Verification
```

Стає:

```text
Problem Statement
↓
Context
↓
Ready Solution lookup
↓
Knowledge / Artifact lookup
↓
Local model inventory check
↓
Compatible model available?
    ├─ yes → local execution candidate
    ├─ no, download allowed → acquisition candidate
    ├─ no, remote allowed → remote capability candidate
    └─ no → negative / open result
↓
Verification
↓
Evidence
```

Головне: **модель не стає початковою точкою**. Початковою точкою лишається Problem Statement.

---

# 11. Що саме треба додати в проєкт

Мінімальний issue-набір:

```text
#48 Model Artifact/Profile schema extension
Done when:
  є JSON schemas для ModelArtifact, ModelProfileArtifact, LocalModelInventoryArtifact.

#49 Local Model Inventory CSU
Done when:
  `aira models scan` створює Local Model Inventory Artifact.

#50 Hardware-to-model compatibility resolver
Done when:
  система класифікує моделі як runnable / downloadable-compatible / incompatible.

#51 Model acquisition policy
Done when:
  auto-download disabled by default; user can allow model downloads by size/trust/scope.

#52 Model download/activation flow
Done when:
  model artifact downloads only after policy check and verifies hash/signature before activation.

#53 Custom model publishing flow
Done when:
  user can publish signed Model Artifact descriptor without forcing global availability.

#54 Model rating evidence
Done when:
  рейтинг є context-specific evidence artifact, not global scalar score.

#55 Upgrade recommendation artifact
Done when:
  система формує evidence-backed рекомендації: model change / quantization / storage / RAM / VRAM / no upgrade.
```

---

# 12. Що не треба робити

```text
Не додавати Model у Core Ontology.
Не робити центральний глобальний Model Registry.
Не робити автоматичне завантаження за замовчуванням.
Не робити один глобальний рейтинг моделей.
Не змушувати всі вузли тримати однаковий набір моделей.
Не змішувати model sharing із settlement/contribution.
Не запускати завантажену модель без explicit activation.
Не робити Recommendation CSU рекламним або marketplace-компонентом.
```

---

# 13. Остаточний висновок

Гіпотеза правильна, але її треба сформулювати так:

```text
AIRA повинна підтримувати розходження локальних модельних наборів як нормальний стан системи.
```

Моделі мають бути:

```text
immutable signed artifacts;
описані через model profiles;
зв’язані з capabilities;
доступні через policy-controlled acquisition;
шарені через artifact publication;
оцінювані через evidence-based contextual rating;
рекомендовані через cost/benefit upgrade analysis.
```

Це не змінює ядро AIRA. Це додає практично необхідний **Model Artifact / Inventory / Acquisition / Sharing** шар у вигляді CSU та Artifact contracts. Саме так проблема вирішується без роздування архітектури.
