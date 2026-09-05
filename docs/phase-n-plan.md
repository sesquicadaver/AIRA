# Phase N — Global Node Rendezvous & Prime Connectivity

**Статус:** складено 2026-09-05; **IN PROGRESS** → QUEUE `#231`–`#247` (RFC-0123 reserved).  
**Джерела:** developer ТЗ *AIRA Global Node Connectivity & Rendezvous* (`aira-prime.md`) + decisions (`aira-current.md`, 2026-09-05); peer stack (`address_book`, DHT, STUN, relay, Noise/Trust); QUEUE M closed @ RFC-0117.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase N `#231`–`#247`.  
**Closure RFC:** `AIRA-RFC-0123` — id **confirmed free** at `#231` (файл лише в `#247`).  
**Не змінює:** анти-місію README (ядро **не** хост інференсу; **не** GPU marketplace; AIRA **не** blockchain network); C0/C1 `Calculate 2 + 2`; `aira-core`; Phase M OS sandbox; AIRA-mediated `network=none`.

> Примітка: `aira-prime.md` / `aira-current.md` — рішення розробника поза git-каноном AIRA. Канон у репо — цей план + QUEUE + RFC.

## 0. Навіщо (після M, не замість peer stack)

Peer stack уже розділяє discovery, trust і dial; `address_book.json` — authoritative dial source. Бракує **глобального zero-knowledge discovery**: дві незалежні ноди без PeerInvite не знають одна про одну.

```text
Phase M OS child sandbox DONE (#224–#230 / RFC-0117)
  → Phase N: global rendezvous + prime-port connectivity
    → не Core ledger; не AIRA consensus; не trust-from-discovery
```

## 1. Мета

Додати механізм, який дозволяє незалежним AIRA-нодам:

1. дізнатися про існування інших нод без попереднього знайомства;
2. знаходити актуальні endpoints;
3. встановлювати direct або relay Noise session;
4. доводити inbound reachability peer-assisted probe;
5. використовувати лише prime ports із Dynamic/Private `49152–65535` для **AIRA-owned** transport.

Цільовий ярлик: **Reference v0.3-strict + optional global rendezvous** (не v0.4; не production multi-chain).

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` **не** отримує ledger/NAT/STUN/relay/port/reachability logic.
3. Distributed ledger = лише rendezvous (ordering + persistence + global lookup); **не** AIRA identity.
4. AIRA Ed25519 signature на Presence Record = authenticity; EVM tx sender лише оплачує publish.
5. `DISCOVERED ≠ TRUSTED`; auto `TrustStore.upsert()` для ledger peer заборонений.
6. AddressBook лишається dial authority; TrustStore — trust authority.
7. AIRA-owned transport endpoints ∈ `P_AIRA` (|P_AIRA|=1491; first=49157; last=65521); fail-closed, без grace.
8. Prime invariant **не** обмежує outbound до Polygon RPC / STUN / HTTP.
9. Public `network_id` = `aira:network:public:v1` (≠ `federation_id`).
10. C1 `Calculate 2 + 2` лишається `execution-basic`.
11. Не редагувати `Manifesto etc/`, `Meditation_About/`.
12. CI: mock `RendezvousProvider` + two-node harness; без обов’язкового Amoy у merge gate.

## 3. Анти-місія (ніколи в QUEUE)

```text
GPU marketplace / compute rental
LLM runtime (Core як inference host)
blockchain network / scheduler **in Core**
AIRA-owned consensus / validators / tokenomics / fork-choice
Core що «вирішує задачі» замість CSU contracts
```

Phase N **є** дозволеним: external EVM ledger як rendezvous substrate + peer connectivity. Це не «зробити AIRA blockchain».

## 3.1. Поза цим етапом

```text
IPFS/HTTP gateway як primary rendezvous adapter
окремий public AIRA probe service
grace/migration для non-prime peer ports
macOS/Windows-specific NAT beyond existing STUN/relay
повний Book II distributed OperationalPlane
model marketplace
вимагати Polygon Amoy у CI merge gate
inventing Phase O
```

## 4. Зафіксовані рішення (aira-current)

| Тема | Рішення |
|------|---------|
| Ledger | `EvmRendezvousProvider`; CI = deterministic local double; reference public = Polygon Amoy `80002`; prod-compatible = Polygon `137` |
| Probe | peer-assisted; перша нода `UNKNOWN`; CI = two-node external-process harness |
| Non-prime | fail-closed одразу; явна діагностика + suggested preferred; без silent rewrite |
| `network_id` | `aira:network:public:v1` |

## 5. Смуги (одна лінія)

```text
N0 govern (wiring)
  → N1 Prime Port invariant
    → N2 deterministic preferred port
      → N3 Presence Record schema/signature
        → N4 RendezvousProvider abstraction
          → N5 EVM ledger adapter (local double)
            → N6 publish/query
              → N7 Reachability Probe
                → N8 Reachability state machine
                  → N9 AddressBook promotion
                    → N10 relay integration
                      → N11 refresh/republication
                        → N12 CLI
                          → N13 Desktop UX
                            → N14 ab ovo integration
                              → N15 NAT/relay integration
                                → N16 RFC-0123 + close
```

## 6. Атоми → QUEUE `#231`–`#247`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#231` | N0 | Phase N wiring **DONE** | QUEUE `#231` DONE; `phase_n_doc` | Prime Port (#232) |
| `#232` | N1 | Prime Port invariant **DONE** | `prime_port`; `|P_AIRA|==1491`; fail-closed; RFC-0124 | preferred selection (#233) |
| `#233` | N2 | Deterministic port selection **DONE** | `preferred_port`; collision walk; RFC-0125 | Presence (#234) |
| `#234` | N3 | Presence Record **DONE** | schema + canonical Ed25519; RFC-0126 | Rendezvous trait (#235) |
| `#235` | N4 | RendezvousProvider | trait API; no ledger-specific in Core | EVM adapter (#236) |
| `#236` | N5 | EVM ledger adapter | local deterministic double; Amoy/mainnet config hooks | publish/query (#237) |
| `#237` | N6 | Publish/query | publish/update/query_active/query_identity; TTL/sequence | Reachability probe (#238) |
| `#238` | N7 | Reachability Probe | peer-assisted signed challenge; no hairpin proof | state machine (#239) |
| `#239` | N8 | Reachability states | UNKNOWN…OFFLINE; `reachability.json` | AddressBook (#240) |
| `#240` | N9 | AddressBook promotion | valid Presence + trust policy → AddressBook only | relay (#241) |
| `#241` | N10 | Relay integration | direct→NAT→relay; prime relay ads; dual reservation SHOULD | refresh (#242) |
| `#242` | N11 | Presence refresh | sequence++; expire stale; endpoint change path | CLI (#243) |
| `#243` | N12 | CLI | `peer port|reachability|rendezvous` commands | Desktop (#244) |
| `#244` | N13 | Desktop UX | Network: port/reachability/rendezvous/peer count | ab ovo (#245) |
| `#245` | N14 | Ab ovo integration | A publish → B discover → trust → dial; no preconfigured peers | NAT/relay (#246) |
| `#246` | N15 | NAT/relay integration | both inbound blocked → relay Noise succeeds | RFC close (#247) |
| `#247` | N16 | RFC-0123 + close | consolidating RFC; QUEUE N closed | — |

```text
#231 N0
  → #232 N1 Prime Port
    → #233 N2 selection
      → #234 N3 Presence
        → #235–#237 N4–N6 Rendezvous
          → #238–#239 N7–N8 Reachability
            → #240–#242 N9–N11 mesh glue
              → #243–#244 N12–N13 UX
                → #245–#246 N14–N15 integration
                  → #247 N16 close
```

## 7. Acceptance Phase N

```text
QUEUE `#231`–`#247` DONE; no OPEN N atoms.
AIRA-owned endpoints use only P_AIRA (1491 primes).
Service ports incl. 443 rejected for AIRA transport.
Preferred port deterministic from identity.
Local bind ≠ external reachability; peer-assisted proof.
First node can publish with reachability UNKNOWN.
Second clean node discovers first via ledger without PeerInvite.
Discovery does not auto-trust.
Trusted peer may promote to AddressBook.
Direct when possible; relay when not.
Ledger outage does not tear existing Noise sessions.
Non-prime endpoints fail-closed.
aira-core has no ledger/network implementation deps.
C1 2+2 still VERIFIED via execution-basic.
Анти-місія не порушена.
```

## 8. RFC

- Consolidating (`#247`): `AIRA-RFC-0123` — id **confirmed free** at `#231`.
- Per-atom RFC-S/RFC-D у `#232`–`#246` (окремі файли; не RFC-A; не RFC-0123 до close).

## 9. Модульна структура (ціль)

```text
aira-peer
    prime_port
    presence
    rendezvous
    reachability
(+ existing address_book / discovery / dht / relay / session / stun)
```

CLI/Desktop лише оркеструють API. Persisted: `peers/rendezvous.json`, `peers/reachability.json`; AddressBook/TrustStore не дублювати.

## 10. Activation rule

```text
QUEUE M closed (RFC-0117) + aira-prime.md + aira-current.md decisions 2026-09-05
  → цей план активовано атомом `#231`.
RFC-0123 file-free until `#247`.
```
