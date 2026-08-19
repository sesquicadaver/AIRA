# Phase C — план стабілізації Reference Implementation

**Статус:** складено 2026-08-19. Виконання — лише через [`QUEUE.md`](../QUEUE.md), перший OPEN, один рядок за цикл.  
**Джерела:** `EVO-1.md`, `EVO-2.md`, Book 0–IV, Conformance, Schema Pack, поточний CI/код.  
**Не канон backlog:** цей файл фіксує загальний план і атомізацію; канон виконання — `QUEUE.md`.

## 0. Порядок роботи

```text
загальний план (цей документ, §1–§3)
  → атомарні задачі без перекриття (§4)
    → лінійний порядок (§5 = хвіст QUEUE #38–#52)
      → виконання: один OPEN рядок = один Analyze-цикл
```

Заборонено додавати поодинокі пункти в чергу поза цим планом. Нова робота після `#52` — [`docs/phase-d-plan.md`](phase-d-plan.md) (Model Artifact & Inventory), не «ще один рядок» у Phase C.

---

## 1. Загальний план

**Мета.** Перевести AIRA з alpha-прототипу в стабільну reference-базу:

```text
Book 0–III = нормативний контракт
Book IV    = референсна реалізація
Book V     = ізольований Research Track
CI         = автоматична перевірка відповідності
```

**Стан входу.** Phase B `#18`–`#37` DONE. Локальний MVP, HTTP, peer/federation-прототипи вже в коді. OPEN до цього плану не було.

**Інваріанти**

- Не редагувати `Manifesto etc/`, `Meditation_About/`.
- Не розширювати Core доменною логікою, GPU/LLM/scheduler, peer-конкретикою.
- Не додавати CRP, новий federation mode, DHT, PHM, HIE, KnowledgeOps, Goal Compiler, DSM, ML optimizer, зовнішній CSU SDK.
- `OperationalPlane` не стає production/distributed runtime.
- Рефакторинг файлів — mechanical only: без нових фіч, без зміни CLI/API/схем і без навмисної зміни поведінки тестів.

**Три контури** (межі для атомізації; не задачі)

| Контур | Мета | Всередині | Зовні |
|--------|------|-----------|--------|
| A. Нормативний gate | CI перевіряє контракт AIRA | schema fixtures, C0, C1 як обов’язковий workflow | семантика схем, новий C2 profile, нові протоколи |
| B. Підпис дескриптора | Підпис = canonical descriptor hash | helper + Event + Artifact + Object + CSU manifest + залишок test-fallback | Noise/mTLS, нові ключі, peer envelope як новий протокол, federation |
| C. Межі й читабельність | Зменшити ентропію без нової поведінки | firewall залежностей; split CLI/crypto/tenant/HTTP/TLS; статус plane; матриця реалізації | нові команди, новий HTTP API, нові CSU |

**Критерій завершення етапу** (усі `#38`–`#52` = DONE)

```text
CI падає на невалідній схемі та на провалі C0/C1.
Зміна полів відповідного дескриптора ламає підпис; runtime без LOCAL_TEST fallback.
aira-core не залежить від aira-node / aira-peer / concrete CSU; CI ловить forbidden imports.
Моноліти CLI/crypto/tenant/HTTP/TLS рознесені без зміни поведінки.
OperationalPlane явно reference-local; є docs/implementation-status.md.
Не додано нових системних сутностей Phase D–G.
```

**Після Phase C:** [`phase-d-plan.md`](phase-d-plan.md) — Model Artifact & Inventory (CSU/Artifact, не Core; не C1-крок). Full local C2 / CSU SDK / research promotion — окремо, не змішувати з D0–D3.

**Фільтр задачі**

1. Чи закриває дірку CI / підпису / меж Core / відповідності Book 0–III?  
2. Чи scope не перетинається з іншим рядком цього плану?  
3. Чи є один вимірюваний Done when?  
4. Чи обходиться без зміни канону Books?

Ні → не в Phase C.

---

## 2. Принцип неперекриття

- **Один файл-контракт — один рядок**, якщо зміна саме цього контракту (CI workflow; один тип дескриптора; один монолітний `.rs`).
- Mutation-тести живуть **у тому ж рядку**, що й перехід цього типу на canonical hash — окремий «тестовий» рядок дублював би B.
- Helper (`#39`) лише додає примітив; **жоден production call-site** не перемикається, доки не дійде свій рядок.
- Split модулів не змінює граф залежностей crates; firewall (`#45`) фіксує граф **до** split, щоб перенесення файлів не маскувало нові ребра.
- Документація статусу (`#51`–`#52`) не чіпає crypto/CI.

---

## 3. Що свідомо не входить

```text
C2 profile як обов’язковий CI
federation leave / Join Request / CRP / HTTP federation API
розширення DHT/relay/gossip
PHM / HIE / Research promotion
зміна Book 0–III
нове CLI/API «для зручності»
```

---

## 4. Атомарні задачі

Нумерація = майбутні рядки QUEUE. Analyze-номери зарезервовані лінійно після Analyze-72.

| ID | Контур | Атом | Done when | Не перекриває / Out |
|----|--------|------|-----------|---------------------|
| `#38` | A | CI: schema fixtures + C0 + C1 як обов’язкові кроки GitHub Actions | Червоний CI при невалідній схемі або провалі C0/C1; зелений на поточному дереві | Зміна семантики схем; C2 у CI; виправлення підпису; split файлів |
| `#39` | B | Примітив canonical descriptor hash + sign/verify **без** зміни call-sites | Тести helper; production verify-шляхи не змінені | Перемикання Event/Artifact/Object/CSU; прибирання fallback |
| `#40` | B | Event: підпис/перевірка = canonical event descriptor без `signature`; mutation-тести; немає event-path runtime fallback на `LOCAL_TEST_DOMAIN_MSG` | Зміна `event_type` / `causal_refs` / `object_refs` / `artifact_refs` / `payload_hash` ламає verify | Artifact/Object/CSU; protocol envelope; Noise |
| `#41` | B | Artifact descriptor: те саме для Artifact | Mutation-тести артефактних полів ламають verify | Event/Object/CSU; CAS layout |
| `#42` | B | Core Object descriptor: те саме для Object | Mutation-тести об’єктних полів ламають verify | Event/Artifact/CSU |
| `#43` | B | CSU manifest signature: canonical manifest без `signature` | Mutation-тести полів маніфесту ламають verify | Event/Artifact/Object; нові CSU |
| `#44` | B | Залишок runtime fallback `LOCAL_TEST_DOMAIN_MSG` / TESTSIG поза `#[cfg(test)]` (envelope, identity, log — лише те, що лишилось після `#40`–`#43`) | Runtime verify не приймає test-domain fallback; тести fixtures — лише test cfg | Повторна зміна вже переведених дескрипторів; нові протоколи |
| `#45` | C | Dependency firewall: `aira-core` ↛ node/peer/concrete CSU; CSU ↛ CSU; цикл імпортів заборонено; перевірка в CI | CI падає на forbidden import / цикл | Split файлів; нові crates |
| `#46` | C | Mechanical split `crates/aira-cli/src/main.rs` → `commands/*` | Поведінка CLI незмінна; тести CLI зелені | crypto/tenant/http/tls; нові підкоманди |
| `#47` | C | Mechanical split `crates/aira-object/src/crypto.rs` | Тести object/crypto зелені | `tenant.rs`; CLI; HTTP |
| `#48` | C | Mechanical split `crates/aira-object/src/tenant.rs` | Тести tenant зелені | `crypto.rs`; HTTP authz семантика |
| `#49` | C | Mechanical split `crates/aira-node/src/http.rs` | HTTP тести/поведінка незмінні | `tls.rs`; нові маршрути |
| `#50` | C | Mechanical split `crates/aira-node/src/tls.rs` | TLS/mTLS/health bind незмінні | `http.rs`; нові режими TLS |
| `#51` | C | Позначити `OperationalPlane` як reference-local-flow (модуль + docs) | Явно: не production event runtime, не scheduler, не federation runtime | Зміна drain/loop семантики |
| `#52` | C | `docs/implementation-status.md`: ТЗ → модуль → тести → статус | Покриття Book 0–IV, schemas, C0/C1, CSU basic, HTTP/peer як post-MVP | Нова реалізація «щоб заповнити матрицю» |

---

## 5. Лінійний порядок

Залежності лише вперед; паралелі немає.

```text
#38 CI gate
  → #39 hash helper
    → #40 Event
      → #41 Artifact
        → #42 Object
          → #43 CSU manifest
            → #44 leftover test-fallback
              → #45 dependency firewall
                → #46 CLI split
                  → #47 crypto split
                    → #48 tenant split
                      → #49 HTTP split
                        → #50 TLS split
                          → #51 reference-local plane
                            → #52 implementation-status
```

**Чому так**

1. `#38` першим: кожна наступна зміна не зможе «позеленіти» без схем і C0/C1.  
2. `#39` до міграцій: один примітив, без подвійної семантики підпису в call-sites.  
3. Event → Artifact → Object → CSU: від найцитованішого P0 (`InvariantChecker`/event log) до маніфесту; типи не змішуються.  
4. `#44` після міграцій: sweep лише залишків, без повторного перепису `#40`–`#43`.  
5. `#45` до split: граф залежностей фіксується, перенесення файлів його не підміняє.  
6. Split від CLI до TLS: різні файли, без спільного diff.  
7. Документація в кінці: фіксує факт після коду, не навпаки.

Виконання: `QUEUE.md` `#38` = Analyze-73, далі +1. Не стартувати `#N+1`, доки `#N` не DONE.
