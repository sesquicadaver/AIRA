# Phase G — Reference completion v0.2

**Статус:** складено 2026-08-24; **OPEN** → QUEUE `#120`–`#146`.  
**Джерела:** [`implementation-status.md`](implementation-status.md) (PARTIAL/STUB/ABSENT); [`phase-f-plan.md`](phase-f-plan.md) §3 post-F; EVO-2 (local C2 → controlled C3 → CSU SDK); [`peer-link.md`](peer-link.md) Out.  
**Не канон backlog:** канон виконання — [`QUEUE.md`](../QUEUE.md).  
**Не змінює:** Book 0–III semantics без RFC; не додає CRP/settlement/PHM/HIE як runtime dependency.

## 0. Порядок

```text
Phase F DONE (#107–#119)
  → цей план (§4 атоми)
    → лінійний хвіст QUEUE #120+
      → один OPEN = один Analyze-цикл
```

Заборонено додавати поодинокі пункти в QUEUE поза цим планом.

## 1. Мета

Підняти reference з **hardened local preview** до **v0.2 reference completion**: глибші local C2 conformance, закриття документованих Book 0 gaps, mechanical de-monolith великих модулів, стабільність Desktop peer lifecycle, мінімальний federation/C3 scaffold, основа CSU SDK і Epistemic basic CSU, production packaging scripts — **без** CRP/settlement/marketplace.

```text
govern (CI truth, living spec)
  → deepen C2 (local protocol cases)
  → close evidence gaps (Claim/VRA)
  → modularize (crypto, peer, cli peer, desktop app)
  → harden desktop peer CI + camera QR
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

## 3. Свідомо не входить (Phase H / RFC-A/B)

```text
CRP (B2-006)
Settlement / Audit protocol (B2-011)
PHM / HIE / KnowledgeOps / Goal Compiler / DSM
production event scheduler / distributed plane
GPU/LLM marketplace / blockchain settlement
full Book II wire network / discv5 production mesh
Universal Intent Theory у Reference
```

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
| `#134` | G5 | Federation leave | CLI `federation leave`; membership cleared; tests |
| `#135` | G5 | Federation join hardening | edge cases (wrong pin, rejoin); audit tests |
| `#136` | G5 | C3 conformance scaffold | `run_c3` minimal local cases; optional CI job doc |
| `#137` | G6 | Event log durability audit | corruption/partial JSON recovery test; doc note |
| `#138` | G6 | SQLite object store path | migration/integrity smoke; doc in implementation-status |
| `#139` | G7 | External CSU fixture | third-party manifest JSON loads in registry smoke |
| `#140` | G7 | CSU SDK doc + conformance | `docs/csu-sdk.md` + named third-party CSU case |
| `#141` | G8 | `epistemic-basic` CSU | `csu/epistemic-basic`; EPI-001 smoke test |
| `#142` | G8 | Epistemic assessment path | plane/CLI smoke: epistemic artifact roundtrip |
| `#143` | G9 | macOS codesign doc+script | `desktop-packaging-macos.md` + notarization helper |
| `#144` | G9 | Windows MSI pipeline | `desktop-packaging-windows.md` + build script |
| `#145` | G9 | Linux deb metapackage | deb/.desktop production path doc + script |
| `#146` | G10 | Phase G docs closure | README + `implementation-status` v0.2; RFC-0069; QUEUE close G |

```text
#120 branch protection
  → #121 living spec
    → #122–#124 C2 cases
      → #125–#126 evidence gaps
        → #127–#130 splits
          → #131–#133 desktop peer
            → #134–#136 federation/C3
              → #137–#138 durable stores
                → #139–#140 CSU SDK
                  → #141–#142 epistemic
                    → #143–#145 packaging
                      → #146 docs RFC
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

- Consolidating atom: [`AIRA-RFC-0069`](../../specs/rfc/AIRA-RFC-0069-phase-g-reference-v0.2.md) (`#146`).
- Per-atom RFC-G лише при контрактній зміні (C3 scaffold, epistemic CSU, federation leave).

## 7. Phase H (preview, не в QUEUE)

```text
CRP protocol design (RFC only)
Settlement ledger prototype
PHM / HIE research promotion (Book V track)
Production distributed OperationalPlane
Full C3–C5 conformance + CI
```

Додавання Phase H — лише новий план + новий хвіст QUEUE після DONE `#146`.
