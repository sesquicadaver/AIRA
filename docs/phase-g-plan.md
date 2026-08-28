# Phase G — Reference completion v0.2

**Статус:** складено 2026-08-24; **DONE** 2026-08-28 → QUEUE `#120`–`#151` (RFC-0069).  
**Джерела:** [`implementation-status.md`](implementation-status.md) (PARTIAL/STUB/ABSENT); [`phase-f-plan.md`](phase-f-plan.md) §3 post-F; EVO-2 (local C2 → controlled C3 → CSU SDK); [`peer-link.md`](peer-link.md) Out.  
**Не канон backlog:** канон виконання — [`QUEUE.md`](../QUEUE.md).  
**Не змінює:** Book 0–III semantics без RFC; не додає CRP/settlement/PHM/HIE як runtime dependency.
**Closure:** [`AIRA-RFC-0069`](../specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md).

## 0. Порядок

```text
Phase F DONE (#107–#119)
  → цей план (§4 атоми)
    → лінійний хвіст QUEUE #120+
      → один OPEN = один Analyze-цикл
```

Заборонено додавати поодинокі пункти в QUEUE поза цим планом.

## 1. Мета

Підняти reference з **hardened local preview** до **v0.2 reference completion**: глибші local C2 conformance, закриття документованих Book 0 gaps, mechanical de-monolith великих модулів, стабільність Desktop peer lifecycle, мінімальний federation/C3 scaffold, основа CSU SDK і Epistemic basic CSU, production packaging scripts — **без** CRP/settlement як product pivot і **без** анти-місії README (GPU marketplace, LLM runtime, blockchain, scheduler).

```text
govern (CI truth, living spec)
  → deepen C2 (local protocol cases)
  → close evidence gaps (Claim/VRA)
  → modularize (crypto, peer, cli peer, desktop app)
  → harden desktop peer CI + camera QR
  → SEC hardening (trust, envelope, identity binding, equivocation, artifact admission)
  → federation minimal (leave, join audit, C3 scaffold)
  → durable stores audit
  → CSU SDK smoke
  → epistemic-basic CSU
  → production packaging (macOS/Windows/Linux)
  → document (Phase G RFC closure)
```

## 2. Інваріанти

1. **Лінійність:** `#N` стартує лише після `#N-1` = DONE.
2. **Один атом = один PR:** не зливати C2 cases, не зливати splits різних crate.
3. **Mechanical split** — без зміни CLI/API/схем і навмисної зміни тестів.
4. C0/C1 gate не послаблюється; C2 job залишається додатковим.
5. Core ↛ node/peer/CSU; CSU ↛ CSU — firewall `#45` обов’язковий.
6. `OperationalPlane` — reference-local; не production distributed runtime.
7. Peer/DHT/relay/gossip — лише стабілізація і документовані edge cases; не discv5/ICE/TURN.

## 3. Анти-місія AIRA (ніколи в Reference, не в QUEUE)

Канон: README §«What AIRA is not»; Book I Core boundaries. **Не** відкладені фази — **заборона продукту/архітектури.**

```text
GPU marketplace / GPU rental / compute marketplace
LLM runtime (Core як inference host)
blockchain network / blockchain-required settlement
scheduler / distributed job orchestration (Core)
Core що «вирішує задачі» замість CSU contracts
```

У лінійній черзі й планах етапів **не з’являються** як «Phase H», «backlog» або «later». Колонка «Не в цьому рядку» у QUEUE = той самий клас відсічення (напр. `#68` marketplace, `#139` CSU marketplace).

## 3.1. Поза scope Phase G / лише Book V або RFC-A/B

Відкладені **в межах місії AIRA** (Problem→VRA, protocols, CSU), але не в `#120`–`#146`:

```text
CRP (B2-006) — protocol spec; не marketplace
Settlement / Audit protocol (B2-011) — не blockchain ledger product
PHM / HIE / KnowledgeOps / Goal Compiler / DSM — Book V / Research track
production event scheduler / distributed OperationalPlane
full Book II wire network / discv5 production mesh
Universal Intent Theory у Reference
```

Додавання в QUEUE — лише новий **план етапу** + RFC, без анти-місії з §3.

## 4. Атоми → QUEUE

| ID | Підфаза | Атом | Done when |
|----|---------|------|-----------|
| `#120` | G0 | CI branch protection sync | `docs/ci-governance.md` + contract test: `conformance-c2` у required checks; checklist для GitHub UI |
| `#121` | G0 | Living spec smoke | script або test: C0 case ids з `implementation-status.md` існують у `run_c0` |
| `#122` | G1 | C2 event idempotency case | conformance `c2.event.publish_idempotent`; C2 job green |
| `#123` | G1 | C2 artifact hash mismatch | conformance `c2.artifact.hash_mismatch`; tampered hash → fail |
| `#124` | G1 | C2 envelope unsigned reject | conformance `c2.protocol.envelope_unsigned`; unsigned → fail |
| `#125` | G2 | Claim/Assumption fixtures | valid/invalid JSON + `schema validate --fixtures`; B0-005 partial closed |
| `#126` | G2 | VRA extended fields | conformance `c1.result.extended_fields` або розширення `verified_completeness` |
| `#127` | G3 | Split `aira-object/crypto/mod.rs` | mechanical modules; `cargo test -p aira-object` green |
| `#128` | G3 | Split `aira-peer/src/lib.rs` | mechanical modules; `cargo test -p aira-peer` green |
| `#129` | G3 | Split `aira-cli/commands/peer.rs` | mechanical modules; CLI peer tests green |
| `#130` | G3 | Split `aira-desktop/src/app.rs` | mechanical modules; desktop tests green |
| `#131` | G4 | Stabilize `peer_lifecycle_p3` | serial/retry pattern як #118; CI stable |
| `#132` | G4 | Stabilize `peer_lifecycle_p4` | serial/retry pattern; CI stable |
| `#133` | G4 | Invite QR camera | GUI capture → import invite; roundtrip smoke |
| `#134` | G-SEC | SEC-1 local-test trust hardening | runtime trust без `local-test`; peer reject; negative test |
| `#135` | G-SEC | SEC-2 canonical envelope signature | full envelope/response sign; C2 field mutation tests |
| `#136` | G-SEC | SEC-3 producer identity binding | `signature.key_ref == producer_identity`; store rejects |
| `#137` | G-SEC | SEC-4 event equivocation | same id + diff hash → conflict; C2 case |
| `#138` | G-SEC | SEC-5 artifact admission | immutable descriptor admission; supersession persist |
| `#139` | G5 | Federation leave | CLI `federation leave`; membership cleared; tests |
| `#140` | G5 | Federation join hardening | edge cases (wrong pin, rejoin); audit tests |
| `#141` | G5 | C3 conformance scaffold | `run_c3` minimal local cases; optional CI job doc |
| `#142` | G6 | Event log durability audit | corruption/partial JSON recovery test; doc note |
| `#143` | G6 | SQLite object store path | migration/integrity smoke; doc in implementation-status |
| `#144` | G7 | External CSU fixture | third-party manifest JSON loads in registry smoke |
| `#145` | G7 | CSU SDK doc + conformance | `docs/csu-sdk.md` + named third-party CSU case |
| `#146` | G8 | `epistemic-basic` CSU | `csu/epistemic-basic`; EPI-001 smoke test |
| `#147` | G8 | Epistemic assessment path | plane/CLI smoke: epistemic artifact roundtrip |
| `#148` | G9 | macOS codesign doc+script | `desktop-packaging-macos.md` + notarization helper |
| `#149` | G9 | Windows MSI pipeline | `desktop-packaging-windows.md` + build script |
| `#150` | G9 | Linux deb metapackage | deb/.desktop production path doc + script |
| `#151` | G10 | Phase G docs closure | README + `implementation-status` v0.2; RFC-0069; QUEUE close G |

```text
#120 branch protection
  → #121 living spec
    → #122–#124 C2 cases
      → #125–#126 evidence gaps
        → #127–#130 splits
          → #131–#133 desktop peer
            → #134–#138 SEC hardening
              → #139–#141 federation/C3
                → #142–#143 durable stores
                  → #144–#145 CSU SDK
                    → #146–#147 epistemic
                      → #148–#150 packaging
                        → #151 docs RFC
```

## 5. Acceptance Phase G

```text
C2 conformance: ≥8 named local cases; CI regression green.
Book 0 B0-005 / VRA gaps: fixtures або conformance named tests.
Найбільші monoliths (crypto, peer lib, cli peer, desktop app): split без behavior change.
Desktop P3/P4 lifecycle: стабільні на CI.
Federation leave + join hardening + C3 scaffold (local only).
CSU SDK + epistemic-basic: smoke + doc.
Packaging: macOS/Windows/Linux production scripts documented.
implementation-status відображає v0.2 posture; Phase G RFC closure.
```

## 6. RFC

- Consolidating atom: [`AIRA-RFC-0069`](../../specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md) (`#151`).
- Per-atom RFC-G лише при контрактній зміні (C3 scaffold, epistemic CSU, federation leave).

## 7. Post-G → Phase H (один лінійний план)

Після DONE `#151` напрями нижче **зведені** в єдиний лінійний етап (без вилок / паралельних планів):

[`phase-h-plan.md`](phase-h-plan.md) → QUEUE `#152`–`#183`.

Порядок залежності (Conformance C3→C4→C5):

```text
durable stores
  → C3 federation depth + capability advertisement
    → CRP local (B2-006; не marketplace)
      → Settlement audit receipts (B2-011; не blockchain)
        → Research promotion discipline (Book V → RFC-P)
          → docs / RFC-0077 / Reference v0.3
```

Анти-місія з §3 **ніколи** в QUEUE.
