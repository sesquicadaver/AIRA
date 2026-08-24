Провів аналіз `sesquicadaver/AIRA` за поточним станом репозиторію через GitHub. Локально `cargo test` не запускав: аналіз нижче базується на структурі, коді, документації, CI і черзі задач у репо.

## Короткий висновок

Проєкт уже не є лише документацією. Це робочий Rust workspace із реалізованим локальним MVP-пайплайном, базовими CSU, схемами, conformance-шаром, CLI/node, HTTP API та вже частково post-MVP peer/federation/trust функціональністю.

Архітектурно репо загалом тримається канону: **Problem Statement → Context → Reuse/Reduction → Execution only if needed → Verification → Verified Result Artifact → Evidence**. README прямо фіксує цю формулу і забороняє трактувати AIRA як GPU marketplace, LLM runtime, blockchain network або scheduler. 

Головна проблема зараз не в концепції, а в **контролі складності**: код уже почав розростатися швидше, ніж CI/conformance і модульні межі.

---

# 1. Поточний стан проєкту

## 1.1 Репозиторій структурований правильно

Є:

* `specs/` з Book 0–V, conformance, schema-pack, RFC process, MVP roadmap, bootstrap plan; 
* `crates/` з Rust workspace: core, object, event, artifact, policy, csu, flow, schema, conformance, protocol, peer, node, cli; 
* `csu/` з базовими CSU: context, reduction, execution, verification, evidence, artifact; 
* `schemas/` з доменами для common/core/artifact/event/policy/capability/csu/execution/evidence/result/conformance/protocol/identity; 
* `docs/` з crypto, local-node, peer-link, conformance, release notes, systemd runbook. 

Це відповідає Repository Bootstrap Plan і не виглядає як хаотичний набір файлів.

## 1.2 Workspace вийшов за межі початкового MVP

У `Cargo.toml` уже є не тільки MVP crates, а й `aira-protocol`, `aira-peer`, `aira-flow`, `aira-node`, `aira-cli`, а також TLS/Noise/X25519/SQLite/HTTP-залежності. 

Це нормально, бо `QUEUE.md` показує закриті Phase B–F і **активну** Phase G `#120`–`#146` (перший OPEN = `#120`). Проєкт — **після MVP Alpha** і **після Desktop Developer Preview**, у фазі Reference v0.2 completion. 

Тобто проект фактично перебуває **після MVP Alpha і Phase E Desktop**, з активною лінійною чергою Phase G.

---

# 2. Відповідність канону AIRA

## 2.1 Book 0–III зафіксовані

Book 0 задає місію як мікроядерну федеративну систему координованого зменшення невизначеності, яка перетворює Problem Statement на Verified Result Artifact з мінімальною сукупною вартістю. 

Book I правильно обмежує Core: Object Model, Stable ABI, Event Runtime, Policy Gate, Invariant Checker, Security Boundary, CSU Runtime. Також прямо забороняє Core містити LLM/GPU/scheduler/routing/ML/blockchain/knowledge/domain logic. 

Book II визначає протокольний шар як interoperability contracts, не алгоритми. 

Book III формалізує CSU як ізольовані, замінні функціональні одиниці, що працюють поверх ABI через Events/Artifacts/Policy. 

Це узгоджено.

## 2.2 Reference / Research розведені

Book IV прямо визначає Reference Implementation як мінімальний runnable AIRA node без ML, GPU, blockchain, federation, PHM, routing optimizer, distributed execution або Research Plane. 

Book V явно робить Research Track non-normative і забороняє йому змінювати Core, обходити ABI/Policy, створювати Operational Artifact напряму або впливати на Operational Plane без promotion. 

Це правильна межа. Її треба берегти, бо `aira-peer` і federation-прототип уже почали тягнути проект за межі локального MVP.

---

# 3. Реалізація: що зроблено добре

## 3.1 Core мінімальний

`aira-core` містить Object Store, Invariant Checker, SQLite store і core errors. `lib.rs` прямо зазначає, що Core не містить domain/ML/GPU/scheduling logic. Є тести на створення/читання об’єктів, in-place mutation rejection, SQLite persistence, duplicate insert і policy-deny invariant emission. 

Це добре відповідає Book I.

## 3.2 Invariant Checker є, але ще не повний

`InvariantChecker` уже перевіряє object/artifact immutability, event signature, policy-before-action, artifact hash, і створює `InvariantViolation` events. 

Це правильний напрямок. Але див. критичне зауваження по signature binding нижче.

## 3.3 Operational flow реально працює на рівні коду

`aira-flow` з’єднує Problem submit → CSU pipeline → Verified Result / Evidence. Тести покривають:

* створення Problem object і `ProblemSubmitted`;
* demo `Calculate 2 + 2`;
* Ready Solution reuse без Execution;
* failure-to-evidence;
* normative split stub без auto-collapse;
* локальний init/session/result/artifact/event flow;
* YAML/JSON config fail-closed поведінку.  

Це найсильніша частина проєкту: MVP-логіка не лише описана, а реально змодельована.

## 3.4 CSU runtime реалізований у правильній парадигмі

`aira-csu` має manifest, registry, lifecycle, runtime, event dispatch, failure events, publisher identity tests, ABI checks, active/suspended state behavior. 

Це відповідає Book III: CSU ізольовані, замінні, подієві.

## 3.5 Execution CSU обмежений безпечно

`execution-basic` підтримує лише `math.eval.safe`, `text.echo`, `text.uppercase`; явно відхиляє `shell`, `bash`, `network.fetch`, а також capsule constraints `shell=true` і `network != none`. 

Це правильна MVP-межа. Execution не повинен перетворюватися на shell runner.

## 3.6 Protocol layer уже частково C2-ready

`aira-protocol` реалізує envelope, event adapter, artifact adapter, identity, discovery, federation. Тести перевіряють schema-valid envelope, unsigned rejection, response/identity schema, idempotent event publish, unsupported version, artifact hash validation, discovery by Capability rather than Node. 

Це добрий крок, але повний C2/C3 не повинен змішуватися з MVP acceptance.

---

# 4. Основні проблеми

## P0 — CI не перевіряє заявлений build contract

README вказує build-команди:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p aira-cli -- schema validate --fixtures fixtures
```

і також показує conformance C0/C1 commands. 

Але GitHub Actions CI зараз виконує лише:

```text
cargo fmt
cargo clippy
cargo test
```

без schema fixture validation і без conformance C0/C1. 

Це головний дефект контролю якості. Для AIRA саме schema/conformance — нормативний контракт. Якщо CI їх не запускає, репо може “успішно зеленіти”, але відходити від стандарту.

**Висновок:** CI потрібно привести до README/Conformance contract: schema fixtures + C0/C1 conformance мають бути обов’язковими.

---

## P0 — Signature verification недостатньо жорстко прив’язує подію до дескриптора

У `InvariantChecker::check_event_signature()` перевірка пробує верифікувати підпис по `event.payload_hash.as_str().as_bytes()`, а якщо не вийшло — fallback на `LOCAL_TEST_DOMAIN_MSG`. 

Проблеми:

1. Підпис по одному `payload_hash` не очевидно захищає `event_type`, `object_refs`, `artifact_refs`, `policy_refs`, `causal_refs`, `created_at`.
2. Fallback на `LOCAL_TEST_DOMAIN_MSG` допустимий для fixtures/tests, але небезпечний для runtime, якщо він доступний у production path.
3. Schema Pack вимагає canonical serialization / hash-signing discipline для машинно перевірних контрактів, а не “presence signature” як робочий режим. 

**Висновок:** потрібно перевести підпис Events/Artifacts/Core Objects на canonical descriptor hash. `LOCAL_TEST_DOMAIN_MSG` має бути `#[cfg(test)]` або fixture-only compatibility path, не runtime fallback.

---

## P1 — Монолітні файли вже порушують Lightweight doctrine

Є дуже великі файли:

* `crates/aira-cli/src/main.rs` ≈ 95 KB;
* `crates/aira-object/src/crypto.rs` ≈ 99 KB;
* `crates/aira-object/src/tenant.rs` ≈ 58 KB;
* `crates/aira-node/src/http.rs` ≈ 31 KB;
* `crates/aira-node/src/tls.rs` ≈ 23 KB.   

Для раннього alpha це прийнятно. Для AIRA — ризик. Проєкт сам декларує радикальну мінімальність, ізоляцію, один компонент — одна відповідальність. Якщо CLI/crypto/tenant/http будуть рости далі, архітектурна ентропія почне накопичуватись у “зручних” файлах.

**Висновок:** перший технічний refactor має бути не нова функціональність, а розбиття монолітів:

```text
aira-cli:
  commands/{schema,identity,csu,problem,peer,trust,tenant}.rs

aira-object:
  crypto/{keyring,trust_store,rotation,revocation,tenant_signing,audit}.rs

aira-node:
  http/{router,auth,handlers,models}.rs
  tls/{config,mtls,certs,health_listener}.rs
```

---

## P1 — Post-MVP функціональність обганяє нормативну межу

README уже говорить про post-MVP: Ed25519, Identity Keyring, Plane node signing, Trust, Peer P0–P2, DHT/relay/gossip, federation join, multi-tenant HTTP authz. 

`QUEUE.md` підтверджує, що ці задачі закриті, включно з STUN, UDP announce, iterative FIND_NODE, public HTTP bind opt-in, federation join prototype. 

Це не помилка саме по собі. Але потрібно не втратити межу:

```text
MVP/Core correctness
≠
Peer/Federation experimental expansion
```

Зараз ризик у тому, що `aira-peer` може стати другою “архітектурною віссю” раніше, ніж Core/Schema/Conformance зацементовані CI.

**Висновок:** перед новими network/federation задачами треба зробити stabilization pass: CI, conformance, schema freeze, security signature binding, modularization.

---

## P1 — OperationalPlane поки що є demo-runtime, не production runtime

`OperationalPlane` in-process створює CSU handlers, тримає memory object store, CAS artifact store, memory event log, runtime, seq/run_nonce, ready_solutions. Він drain-ить event queue з safety guard `>256`.  

Це нормальний MVP. Але важливо не видати це за повну runtime-модель:

* немає реального async/backpressure;
* немає durable event runtime як основного source of truth у самій plane;
* `is_normative_split()` — строкова евристика/stub;
* `drain_from()` — demo-loop, не general scheduler/runtime;
* sequence ids залишаються локально-детермінованими, не глобально стабільними.

**Висновок:** `aira-flow` має лишатися reference/demo plane до окремого RFC/Book IV update. Не переносити його прямо в production runtime без event-log-first redesign.

---

## P2 — Документація стала багатою, але потребує pruning active corpus

У репо збережено `Manifesto etc/`, `Meditation_About/`, `analysis/`, `specs/`. README вже маркує Manifesto як immutable canonical sources, Meditation як historical drafts, analysis як analytical layer, specs як working copies. 

Це добре. Але для зовнішнього учасника активний вхід все ще завеликий. Потрібен один “reader path”:

```text
README
→ specs/book-0.md
→ specs/book-i.md
→ specs/book-ii.md
→ specs/book-iii.md
→ specs/mvp-roadmap.md
→ docs/demo.md
→ docs/csu-development.md
```

І окремо “historical/provenance path”.

---

# 5. Що вже можна вважати досягнутим

За фактом репо закриває значну частину MVP PRD:

* локальний Problem Statement submit;
* Context Artifact;
* Reduction / ready solution reuse;
* safe deterministic execution;
* explicit Verification;
* Verified Result Artifact;
* Evidence / Failure Evidence;
* Artifact immutability;
* Event traceability;
* Policy Gate;
* C0/C1 test/conformance foundation;
* CLI / node / HTTP API;
* базова identity/trust/tenant work.   

Архітектурно це вже **не концепт**, а ранній alpha reference implementation.

---

# 6. Пріоритет виправлень

## Негайно

1. **CI:** додати `aira schema validate --fixtures fixtures` і `aira conformance run --profile C0/C1`.
2. **Signature binding:** підписувати canonical descriptor hash; прибрати runtime fallback на `LOCAL_TEST_DOMAIN_MSG`.
3. **Freeze нової функціональності:** не додавати нові network/federation фічі до stabilization pass.

## Далі

4. Розбити `main.rs`, `crypto.rs`, `tenant.rs`, `http.rs`, `tls.rs` на модулі.
5. Винести `OperationalPlane` у чітко позначений `reference-local-flow`, щоб не плутати demo plane з production runtime.
6. Додати conformance artifacts у CI output або хоча б у test fixtures.
7. Сформувати `docs/implementation-status.md`: що реалізовано, що stub, що post-MVP, що research.

---

# Підсумкова оцінка

**Стан:** сильний alpha / early reference implementation.
**Архітектурна відповідність:** добра.
**Головний ризик:** розростання post-MVP мережевого/security коду без достатнього conformance enforcement.
**Головний дефект:** CI не перевіряє schema/conformance, тобто не контролює найважливіший контракт AIRA.
**Наступна правильна дія:** stabilization pass, не нові сутності й не нові фічі.
