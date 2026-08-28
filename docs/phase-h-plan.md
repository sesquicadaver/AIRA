# Phase H — Protocol depth & durable reference v0.3

**Статус:** складено 2026-08-28; `#152`–`#161` **DONE**; перший OPEN = `#162`; хвіст `#162`–`#183`.  
**Джерела:** [`phase-g-plan.md`](phase-g-plan.md) §7 (post-G); [`implementation-status.md`](implementation-status.md) PARTIAL/STUB/ABSENT; Conformance C3→C4→C5 ladder; Book II §10/§15; Book V promotion; EVO-2.  
**Не канон backlog:** канон виконання — [`QUEUE.md`](../QUEUE.md).  
**Не змінює:** Book 0–III semantics без per-atom RFC; анти-місію README.

## 0. Порядок (одна лінія, без вилок)

П’ять post-G тем з §7 Phase G **не** є паралельними планами. Вони зведені в **один** лінійний ланцюг за залежністю Conformance:

```text
C3 = C2 + Federation + CRP + Capability Advertisement
C4 = C3 + Settlement/Audit
C5 = C4 + Evolution/Research Separation + Promotion Pipeline
```

Тому єдина допустима послідовність:

```text
Phase G DONE (#151)
  → H0 govern
    → H1 durable stores          (підлога для event/audit)
      → H2 C3 federation depth   (membership + capability ad)
        → H3 CRP local           (B2-006; capability≠node)
          → H4 settlement audit  (B2-011; receipts, не ledger)
            → H5 research promotion discipline (RFC-P / C5 scaffold)
              → H6 docs + RFC-0077 → Reference v0.3
```

Заборонено: паралельні «треки», опційні гілки «A або B», окремі Phase H1/H2 плани, skip атомів.

## 1. Мета

Підняти Reference v0.2 до **v0.3 protocol-depth reference**: durable event/object paths, глибший local C3, локальний CRP і settlement **як контракти/adapters** (не marketplace / не blockchain), дисципліна promotion для Research — **без** анти-місії і **без** production distributed OperationalPlane.

## 2. Інваріанти

1. **Лінійність:** `#N` стартує лише після `#N-1` = DONE.
2. **Один атом = один PR:** не зливати store+CRP, не зливати CRP+settlement, не зливати C4+C5 runners.
3. C0/C1 gate не послаблюється; C2 job лишається required; C3/C4/C5 — спочатку local/optional, поки окремий атом не зробить інакше.
4. Core ↛ node/peer/CSU; CSU ↛ CSU — firewall обов’язковий.
5. `OperationalPlane` — reference-local; не scheduler / не distributed runtime.
6. CRP маршрутизує **Capability**, не Node; Settlement — **audit receipts**, не blockchain ledger; Research — **non-operational** до promotion.

## 3. Анти-місія (ніколи в QUEUE / «Phase H»)

```text
GPU marketplace / compute rental
LLM runtime (Core як inference host)
blockchain network / blockchain-required settlement
scheduler / distributed job orchestration (Core)
Core що «вирішує задачі» замість CSU contracts
```

## 3.1. Поза цим етапом (після `#183`, лише новий план)

```text
full Book II wire / discv5 production mesh
distributed OperationalPlane / production event scheduler
PHM / HIE / Goal Compiler / DSM як runtime
App Store / enterprise Authenticode beyond helpers
```

## 4. Атоми → QUEUE

| ID | Підфаза | Атом | Done when |
|----|---------|------|-----------|
| `#152` | H0 | Phase H wiring | `docs/README.md` + `QUEUE` + `phase-g` §7 вказують на цей план; living smoke: `#152`–`#183` ids узгоджені |
| `#153` | H0 | C3 governance note | `docs/conformance.md` + `ci-governance.md`: C3 local optional; критерій коли C3 може стати job |
| `#154` | H1 | Event log hash-chain tip | append запис + verify tip; tamper mid-log → detect; тест |
| `#155` | H1 | Event log prefix recovery | partial/corrupt trailing bytes → recover valid prefix (не лише full reset); тест |
| `#156` | H1 | Durable event backend module | окремий durable backend (file chain або SQLite events); unit тести |
| `#157` | H1 | Session wires durable events | `LocalSession` / `init_node` default path пише в durable backend; roundtrip reopen |
| `#158` | H1 | Node/plane Sqlite object path | document + test: object store Sqlite path usable beside memory; no Core→node leak |
| `#159` | H1 | Stores status rollup | `implementation-status` stores/event rows updated (PARTIAL notes reflect H1) |
| `#160` | H2 | Capability advertisement persist | load/save local capability ads; тест |
| `#161` | H2 | C3 capability advertisement case | `c3.capability.advertisement` (або еквівалент) у `run_c3` |
| `#162` | H2 | Federation policy export deny | policy-scoped export/import deny smoke + audit |
| `#163` | H2 | C3 cases ≥6 | `run_c3` ≥6 named local cases; doc |
| `#164` | H2 | Optional `conformance-c3` CI job | workflow job (не required gate) + governance note |
| `#165` | H3 | CRP schema fixtures | route request/candidate fixtures + `schema validate --fixtures` |
| `#166` | H3 | CRP local adapter + RFC | `aira-protocol` in-process CRP; short RFC (contract); capability≠node |
| `#167` | H3 | CRP reject node-keyed route | conformance: route by node id → fail |
| `#168` | H3 | CRP multi-candidate + Policy Gate | ≥2 candidates; bind requires ALLOW; DENY → no bind |
| `#169` | H3 | CRP route events | events for selected / rejected / failure; тест |
| `#170` | H3 | B2-006 C3 case | `c3.crp.route_candidate` green in `run_c3` |
| `#171` | H3 | CRP status PARTIAL | `implementation-status` CRP ABSENT→**PARTIAL** |
| `#172` | H4 | Settlement receipt fixtures | schema fixtures validate; privacy fields present |
| `#173` | H4 | Settlement receipt store | append-only receipts (JSONL або SQLite); verify on read smoke |
| `#174` | H4 | B2-011 privacy smoke | receipt privacy class / redaction rules local test |
| `#175` | H4 | `run_c4` scaffold | minimal C4 cases (receipt emit/verify + link to prior event/route) |
| `#176` | H4 | Settlement status PARTIAL | `implementation-status` Settlement ABSENT→**PARTIAL** |
| `#177` | H5 | RFC-P promotion doc | `docs/rfc-p-promotion.md`: Book V → operational лише через promotion |
| `#178` | H5 | Promotion-candidate fixtures | schema fixtures; invalid without required fields |
| `#179` | H5 | Non-operational until promote | research/promotion artifact rejected as operational input; тест |
| `#180` | H5 | `run_c5` scaffold | research separation + promotion gate local cases |
| `#181` | H5 | Promotion status rollup | `implementation-status` research-promotion notes; C5 row |
| `#182` | H6 | Reference v0.3 docs | README + `implementation-status` **v0.3**; conformance index |
| `#183` | H6 | Phase H RFC-0077 closure | consolidating RFC; QUEUE H closed; no OPEN |

```text
#152–#153 H0
  → #154–#159 H1 stores
    → #160–#164 H2 C3 depth
      → #165–#171 H3 CRP
        → #172–#176 H4 settlement
          → #177–#181 H5 promotion
            → #182–#183 H6 docs
```

## 5. Acceptance Phase H

```text
Durable event path: hash-chain tip + prefix recovery + session reopen.
C3: ≥6 local cases; capability advertisement; optional CI job (non-gate).
CRP: local adapter + B2-006 case; status PARTIAL; capability≠node enforced.
Settlement: receipt store + B2-011 smoke + run_c4 scaffold; status PARTIAL.
Promotion: RFC-P doc + non-operational gate + run_c5 scaffold.
Docs: Reference v0.3; RFC-0077; QUEUE #152–#183 DONE.
Анти-місія README не порушена.
```

## 6. RFC

- Consolidating: `specs/rfc/AIRA-RFC-0077-phase-h-protocol-depth-v0.3.md` (`#183`).
- Per-atom RFC обов’язковий при контрактній зміні: CRP adapter (`#166`), settlement receipts (`#173`/`#175`), promotion gate (`#179`/`#180`).

## 7. Post-H

Лише **новий** план етапу після DONE `#183`. Не відкривати паралельні черги.
