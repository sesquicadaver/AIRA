# Рекомендаційний план еволюції AIRA

**Сніпшот:** складено до Phase F; частина P0 stabilization **виконана** (`#107`–`#119`, 2026-08-24). Активна черга: Phase G [`docs/phase-g-plan.md`](docs/phase-g-plan.md) `#120`–`#146`. Канон виконання: [`QUEUE.md`](QUEUE.md).

Поточний стан: репозиторій уже вийшов за межі чистого MVP. README фіксує AIRA як локальну/федеративну систему `Problem Statement → Verified Result Artifact → Evidence`, а не GPU/LLM/blockchain-платформу (README §«What AIRA is not» — **анти-місія**, не backlog). У статусі згадані post-MVP peer/federation/trust і Desktop Developer Preview (Phase E DONE). 

Отже наступний етап — **не нова функціональність**, а **стабілізація Reference Implementation і захист архітектури від розростання**.

---

## 1. Стратегічна мета наступного етапу

Перевести AIRA з “працюючого alpha-прототипу” в **стабільну reference-базу**, де:

```text
Book 0–III = нормативний контракт
Book IV    = референсна реалізація
Book V     = ізольований Research Track
CI         = автоматична перевірка відповідності
```

Book 0 визначає місію як перетворення `Problem Statement` у `Verified Result Artifact` з мінімальною сукупною вартістю, через повторне використання готових рішень, перевірених знань і мінімально достатньої композиції можливостей.  Book I жорстко обмежує Core: Object Model, Stable ABI, Event Runtime, Policy Gate, Invariant Checker, Security Boundary і CSU Runtime — без LLM/GPU/scheduler/routing/ML/domain logic. 

---

# 2. Пріоритети

## P0 — обов’язково до будь-якої нової фічі

### 2.1 CI має перевіряти не лише Rust, а й AIRA-сумісність

Зараз CI виконує тільки `cargo fmt`, `cargo clippy`, `cargo test`.  Але README уже декларує також `schema validate --fixtures` і conformance C0/C1. 

Потрібно додати в CI:

```bash
cargo run -p aira-cli -- schema validate --fixtures fixtures
cargo run -p aira-cli -- conformance run --profile C0 --out /tmp/aira-c0
cargo run -p aira-cli -- conformance run --profile C1 --out /tmp/aira-c1
```

Conformance Specification прямо визначає, що сумісність перевіряється через Book 0 ontology, Book I Core/ABI, Book II protocols, Book III CSU contracts, Verified Result Artifact, failure-to-evidence behavior і Operational/Evolution separation. 

**Done when:**

```text
CI падає при:
- невалідній схемі;
- порушенні Artifact/Event/Object descriptor;
- провалі C0;
- провалі C1;
- відсутності Evidence для failure.
```

---

### 2.2 Криптографічний підпис має бути прив’язаний до canonical descriptor hash

Schema Pack вимагає canonical JSON, deterministic key ordering і SHA-256 для hashing/signing.  Поточний `InvariantChecker` перевіряє event signature по `payload_hash`, а потім має fallback на `LOCAL_TEST_DOMAIN_MSG`. Це допустимо для fixtures, але не для runtime path.

Потрібно:

```text
Object signature    = sign(canonical_object_descriptor_without_signature)
Event signature     = sign(canonical_event_descriptor_without_signature)
Artifact signature  = sign(canonical_artifact_descriptor_without_signature)
CSU manifest sig    = sign(canonical_manifest_without_signature)
```

`LOCAL_TEST_DOMAIN_MSG` залишити тільки для тестів або legacy fixtures.

**Done when:**

```text
зміна event_type ламає підпис;
зміна causal_refs ламає підпис;
зміна object_refs/artifact_refs ламає підпис;
заміна payload_hash ламає підпис;
runtime не приймає TESTSIG/local-test fallback поза тестовим режимом.
```

---

### 2.3 Заборонити нові network/federation/research фічі до стабілізації

Book IV прямо визначає, що Reference Implementation має довести працездатність без ML, GPU, blockchain, federation, PHM, routing optimizer, distributed execution і Research Plane.  Book V фіксує, що Research Track не є умовою сумісності й не має права впливати на Operational Plane без promotion. 

Тому наступний цикл не повинен додавати:

```text
новий CRP;
новий federation mode;
новий DHT;
новий PHM;
новий HIE;
KnowledgeOps;
Goal Compiler;
DSM;
ML optimizer.
```

---

# 3. P1 — стабілізація архітектури коду

## 3.1 Розбити монолітні файли без зміни поведінки

Поточний код уже має великі файли: `aira-cli/src/main.rs`, `aira-object/src/crypto.rs`, `aira-object/src/tenant.rs`, `aira-node/src/http.rs`, `aira-node/src/tls.rs`. Це не помилка alpha-етапу, але вже ризик архітектурної ентропії.

Рефакторинг має бути **mechanical only**:

```text
без нових фіч;
без зміни CLI;
без зміни API;
без зміни схем;
без зміни поведінки тестів.
```

Рекомендований поділ:

```text
crates/aira-cli/src/
  commands/
    schema.rs
    identity.rs
    csu.rs
    problem.rs
    peer.rs
    trust.rs
    tenant.rs

crates/aira-object/src/crypto/
  keyring.rs
  trust_store.rs
  rotation.rs
  revocation.rs
  tenant_signing.rs
  audit.rs

crates/aira-node/src/http/
  router.rs
  auth.rs
  handlers.rs
  models.rs

crates/aira-node/src/tls/
  config.rs
  mtls.rs
  certs.rs
  health.rs
```

---

## 3.2 Зафіксувати dependency firewall

Правило:

```text
core crates не залежать від node/peer/protocol конкретики;
core не імпортує concrete CSU;
CSU не викликають інші CSU напряму;
взаємодія лише через ABI/Event/Artifact/Policy.
```

Book III визначає CSU як ізольовані, замінні функціональні одиниці, які працюють через Stable ABI, Events, Artifacts і Policy, не змінюють Core і можуть бути замінені сумісним CSU. 

**Done when:**

```text
cargo-level dependency graph не має циклів;
aira-core не залежить від aira-node / aira-peer / concrete csu;
concrete CSU не імпортують один одного;
forbidden imports перевіряються в CI.
```

---

## 3.3 Чітко позначити `OperationalPlane` як reference-local-flow

`aira-flow` уже доводить важливі сценарії: submit problem, `Calculate 2 + 2`, Ready Solution reuse, failure-to-evidence, normative split stub, local init/session/result/artifact/event.  

Але це ще не production runtime. Це reference/demo plane.

Потрібно явно зафіксувати:

```text
OperationalPlane = локальна референсна демонстрація C1
не production event runtime
не distributed runtime
не scheduler
не federation runtime
```

---

# 4. P2 — документація і corpus control

## 4.1 Створити один активний reader path

Зараз репозиторій має `Manifesto etc/`, `Meditation_About/`, `analysis/`, `specs/`, `docs/`. Це корисно для provenance, але складно для входу.

Активний шлях має бути таким:

```text
README.md
↓
specs/book-0.md
↓
specs/book-i.md
↓
specs/book-ii.md
↓
specs/book-iii.md
↓
specs/book-iv.md
↓
specs/conformance.md
↓
specs/schema-pack.md
↓
docs/demo.md
↓
docs/csu-development.md
```

Specification Control Document уже визначає, що активний корпус не повинен містити дублікати, а застарілі матеріали мають переходити в `Superseded`, `Historical` або `Research Archive`. 

---

## 4.2 Додати `docs/implementation-status.md`

Не як нову архітектуру, а як карту відповідності:

```text
Requirement
Source spec
Implemented in
Tested by
Status
Notes
```

Приклад:

```text
Artifact immutability
Book I / Schema Pack
crates/aira-artifact
artifact immutability tests
DONE
```

Це зніме головний ризик: різні частини системи вже реалізовані нерівномірно, а статус видно лише через README/QUEUE/tests.

---

# 5. Рекомендована нова черга задач

Нові задачі додаються **лише в кінець OPEN-хвоста** через новий план етапу (див. Phase G [`docs/phase-g-plan.md`](docs/phase-g-plan.md)), зберігаючи правило «один рядок = один Analyze-cycle». Анти-місія README (GPU marketplace, LLM runtime, blockchain, scheduler) **ніколи** входить у QUEUE. 

```text
#38 CI schema/conformance gate
Done when:
  GitHub Actions запускає schema fixtures + C0 + C1.

#39 Canonical descriptor signatures
Done when:
  Object/Event/Artifact/CSU signatures прив’язані до canonical descriptor hash.

#40 Signature mutation tests
Done when:
  зміна event_type / causal_refs / artifact_refs / payload_hash ламає verify.

#41 Core dependency firewall
Done when:
  CI перевіряє заборонені залежності core → node/peer/concrete CSU.

#42 Modularize aira-cli main.rs
Done when:
  CLI розбитий на command modules; behavior unchanged; tests pass.

#43 Modularize crypto/tenant code
Done when:
  keyring/trust/rotation/revocation/tenant_signing/audit розділені; tests pass.

#44 Modularize node HTTP/TLS
Done when:
  router/auth/handlers/models і tls/config/mtls/certs/health розділені; tests pass.

#45 Mark OperationalPlane as reference-local-flow
Done when:
  docs + module comments явно фіксують non-production статус.

#46 Implementation status matrix
Done when:
  docs/implementation-status.md покриває Book 0–IV, schemas, conformance, CSU.

#47 C2 conformance hardening
Done when:
  local Event/Artifact/Identity/Discovery protocol tests покриті conformance profile.
```

---

# 6. Еволюційна дорожня карта після стабілізації

## Phase C — Reference hardening

Мета:

```text
зробити C0/C1 безумовно стабільними;
усунути crypto/test shortcuts;
зменшити моноліти;
зафіксувати статус реалізації.
```

Жодних нових системних сутностей.

---

## Phase D — Full local C2

Мета:

```text
Event Protocol;
Artifact Protocol;
Identity Protocol;
Discovery Protocol;
Protocol Envelope;
Protocol Response;
unsupported-version semantics;
signature semantics;
idempotency.
```

`aira-protocol` уже має основу для envelope, event/artifact adapters, identity, discovery і capability-not-node discovery. 

---

## Phase E — Controlled C3 federation

Тільки після Phase C/D.

Мета:

```text
мінімальна federation compatibility;
policy-scoped import/export;
trust boundary;
no hidden global state;
no mandatory network for Core.
```

Не розширювати DHT/relay/gossip, доки C2 не стабільний.

---

## Phase F — CSU SDK / external CSU

Тільки після стабільного C2.

Мета:

```text
зовнішній CSU manifest;
capability declaration;
sandbox permissions;
event subscriptions;
artifact I/O contract;
conformance fixture for third-party CSU.
```

Це практичніше, ніж ранній HIE/PHM/Research.

---

## Phase G — Research promotion discipline

Будь-яка PHM, HIE, KnowledgeOps, DSM, Goal Compiler, Capability Algebra — тільки через Book V і RFC-P. RFC process уже вимагає problem statement, compatibility impact, rollback, conformance tests і evidence для зміни стандарту. 

---

# 7. Що не робити зараз

```text
Не додавати новий HIE.
Не додавати PHM як runtime dependency.
Не додавати ML optimizer.
Не додавати Goal Compiler.
Не додавати Universal Intent Theory у Reference.
Не розширювати federation до стабілізації C2.
Не переносити peer/federation код у Core.
Не трактувати OperationalPlane як production runtime.
Не додавати нові сутності в Book 0–III без RFC-A/B.
```

Причина проста: MVP PRD визначає першу реалізацію як локальну систему з Problem Submission, Context, Reuse/Reduction, safe execution, Verification, Evidence, immutable Artifacts, Event traceability, Policy Gate і C0/C1 conformance — без real federation, multi-node routing, GPU, LLM, blockchain settlement, PHM або Research Plane. 

---

# 8. Критерій правильності подальшої еволюції

Кожна нова задача повинна проходити фільтр:

```text
1. Чи виправляє вона порушення Book 0–III?
2. Чи покращує вона conformance?
3. Чи зменшує ризик Core pollution?
4. Чи зменшує складність або локалізує її?
5. Чи має вимірюваний Done when?
6. Чи може бути реалізована як CSU/Artifact/Protocol extension замість зміни Core?
```

Якщо відповідь “ні” — задача не входить у Reference queue.

---

## Підсумок

Найкраща наступна еволюція AIRA:

```text
stabilize
↓
verify
↓
modularize
↓
document actual status
↓
harden C2
↓
only then expand federation/research
```

Не треба зараз створювати нові архітектурні шари. Поточна система вже має достатньо концептуальної сили. Її слабке місце — не нестача ідей, а потреба в жорсткому conformance/security/complexity control.
