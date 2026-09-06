# Phase N-fix — Semantic honesty & live rendezvous gaps

**Статус:** складено 2026-09-06; **DONE** @ RFC-0139 — QUEUE `#248`–`#254` **DONE**; **QUEUE N-fix closed** (QUEUE N closed @ RFC-0123 лишається).  
**Джерела:** post-N audit `aira-current.md` @ `43bf3fa` (2026-09-06); Phase N [`phase-n-plan.md`](phase-n-plan.md) **DONE** @ RFC-0123 (local reference).  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase N-fix `#248`–`#254`.  
**Closure:** honesty RFC-D на останньому атомі + статус `PARTIAL→` live global там, де Done when вимагає; consolidating id **RFC-0139** (file at `#254`).  
**Не змінює:** анти-місію README; C0/C1 `Calculate 2 + 2`; `aira-core` ledger deps; QUEUE N історичний DONE @ RFC-0123 (архітектура залишається).

> Примітка: `aira-current.md` поза git-каноном. Канон у репо — цей план + QUEUE + RFC.

## 0. Навіщо (після N, не нова функціональна фаза)

Phase N закрила **локальний reference** (Prime Port → Presence → rendezvous abstraction → reachability model → trust-gated dial/relay).  
Аудит: назва **Global Node Rendezvous** випереджає код — немає live EVM JSON-RPC; ab ovo / NAT тести = loopback semantics; reachability attestation не прив’язана до inbound session.

```text
Phase N DONE (#231–#247 / RFC-0123) = architecture + local reference
  → Phase N-fix: close semantic gaps; do not invent Phase O features
```

## 1. Мета

Зробити Phase N **чесною щодо глобальності** і підтягнути fail-closed інваріанти:

1. Live EVM rendezvous (реальний JSON-RPC read/write), не лише local double.
2. Real ab ovo harness (два зовнішні процеси / незалежні roots).
3. ReachabilityAttestation bound до реального inbound session/transcript.
4. Expiry enforcement перед discovery → AddressBook promotion.
5. NAT/firewall integration через network namespaces (не non-listening loopback).
6. Коректний wrap у `next_candidate_port`.
7. Fundamental temporal Presence invariant у `validate_shape`.

Цільовий ярлик: **Reference v0.3-strict + honest global rendezvous path** (не marketplace; не Core ledger).

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` **не** отримує ledger/NAT/JSON-RPC implementation.
3. `DISCOVERED ≠ TRUSTED`; auto TrustStore upsert заборонений.
4. AddressBook = dial authority; TrustStore = trust authority.
5. AIRA-owned endpoints ∈ `P_AIRA`; fail-closed.
6. C1 `Calculate 2 + 2` лишається `execution-basic`.
7. CI merge gate **не** вимагає live Polygon mainnet; Amoy/local anvil-style opt-in + mock path лишаються.
8. Не редагувати `Manifesto etc/`, `Meditation_About/`.
9. Не переписувати історію QUEUE N DONE; фікси = новий хвіст.

## 3. Анти-місія / поза етапом

```text
GPU marketplace / LLM-in-Core
AIRA-owned consensus / tokenomics
Central AIRA bootstrap as required dependency
Desktop default non-loopback bind
Rewriting Phase N atoms as never-DONE
Full CGNAT field trial as CI required
```

## 4. Смуги

```text
NF0 Live EVM JSON-RPC
  → NF1 Real ab ovo two-process harness
    → NF2 Reachability session/transcript bind
      → NF3 Presence expiry before promote
        → NF4 netns NAT/firewall integration
          → NF5 next_candidate_port wrap fix
            → NF6 Presence created_at < expires_at + honesty close (RFC-0139)
```

## 5. Атоми → QUEUE `#248`–`#254`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#248` | NF0 | Live EVM rendezvous | **DONE** @ RFC-0140: live JSON-RPC publish/query (`anvil_live` / Amoy / `ReferenceEvmRendezvousRpc`) | two-process ab ovo (#249) |
| `#249` | NF1 | Real ab ovo harness | **DONE** @ RFC-0141: two OS processes / independent roots / shared ledger / empty book B | session reachability (#250) |
| `#250` | NF2 | Reachability session bind | **DONE** @ RFC-0142: inbound Noise transcript bind; signed claim без connect ≠ `DIRECT_REACHABLE` | expiry (#251) |
| `#251` | NF3 | Expiry before promote | **DONE** @ RFC-0143: expired Presence fail-closed (promote + discover even via `query_identity`) | netns NAT (#252) |
| `#252` | NF4 | netns NAT/firewall tests | **DONE** @ RFC-0144: firewall DROP ≠ non-listening placeholder; Docker smoke or documented path | port wrap (#253) |
| `#253` | NF5 | `next_candidate_port` wrap | **DONE** @ RFC-0145: modular ring wrap; cur-before-preferred after wrap fixed | Presence shape (#254) |
| `#254` | NF6 | Presence temporal + close | **DONE** @ RFC-0139: `created_at < expires_at`; global live **PARTIAL**; QUEUE N-fix closed | — |

```text
#248 NF0 Live EVM
  → #249 NF1 ab ovo harness
    → #250 NF2 reachability bind
      → #251 NF3 expiry
        → #252 NF4 netns NAT
          → #253 NF5 port wrap
            → #254 NF6 temporal + RFC-0139 close
```

## 6. Acceptance Phase N-fix

```text
QUEUE `#248`–`#254` DONE; no OPEN N-fix atoms.
Live EVM path exists (CI: anvil/local RPC or gated Amoy).
Two-process ab ovo harness green.
DIRECT_REACHABLE requires session-bound attestation.
Expired Presence cannot promote to AddressBook.
next_candidate_port wrap correct.
Presence validate_shape enforces created_at < expires_at.
Docs: Global rendezvous honesty matches code.
C1 2+2 still VERIFIED via execution-basic.
aira-core still has no ledger deps.
Анти-місія не порушена.
```

## 7. RFC

- Per-atom RFC-D: `#248`→[`AIRA-RFC-0140`](../specs/rfc/AIRA-RFC-0140-live-evm-rendezvous.md); `#249`→[`AIRA-RFC-0141`](../specs/rfc/AIRA-RFC-0141-ab-ovo-two-process.md); `#250`→[`AIRA-RFC-0142`](../specs/rfc/AIRA-RFC-0142-reachability-session-bind.md); `#251`→[`AIRA-RFC-0143`](../specs/rfc/AIRA-RFC-0143-presence-expiry-before-promote.md); `#252`→[`AIRA-RFC-0144`](../specs/rfc/AIRA-RFC-0144-inbound-firewall-honesty.md); `#253`→[`AIRA-RFC-0145`](../specs/rfc/AIRA-RFC-0145-next-candidate-port-wrap.md).
- Consolidating: **[`AIRA-RFC-0139`](../specs/rfc/AIRA-RFC-0139-phase-n-fix-honesty-close.md)** at `#254` (**QUEUE N-fix closed**).
- RFC-0123 лишається історичним Phase N closure (local reference).

## 8. Activation rule

```text
QUEUE N-fix closed @ RFC-0139 (2026-09-06).
`#248`–`#254` DONE; no OPEN N-fix atoms.
Global live rendezvous status remains PARTIAL (honest).
Next: Phase O [`phase-o-plan.md`](phase-o-plan.md) `#255`–`#265`.
```
