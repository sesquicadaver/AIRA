# Phase F — Post-E Stabilization v0.1

**Статус:** складено 2026-08-22; **OPEN** → QUEUE `#107`–`#119`.  
**Джерела:** [`phase-e-plan.md`](phase-e-plan.md) §«Пізніше»; [`implementation-status.md`](implementation-status.md); `EVO-1.md` / `EVO-2.md` (stabilize → verify → modularize → harden C2).  
**Не канон backlog:** канон виконання — [`QUEUE.md`](../QUEUE.md).  
**Не змінює:** Book 0–IV semantics; peer protocol; нові network/federation фічі.

## 0. Порядок

```text
phase-e-plan §Пізніше + implementation-status gaps
  → цей план (§4 атоми)
    → лінійний хвіст QUEUE #107–#119
      → один OPEN = один Analyze-цикл
```

Заборонено додавати поодинокі stabilization-пункти в QUEUE поза цим планом.

## 1. Мета

Підняти reference з **Developer Preview** до **hardened reference**: reproducible CI, security/docs truth, verify-on-read, dispatch policy enforcement, mechanical de-monolith, C2 regression gate — **без** нових distributed features.

```text
stabilize (toolchain, CI governance, schema CI)
  → verify (Handle, verify-on-read, policy dispatch, acquisition audit)
  → modularize (model-acquisition split)
  → harden C2 + flaky CI tests
  → document (SECURITY, README, Phase F RFC)
```

## 2. Інваріанти

1. **Без нових фіч** peer/federation/marketplace/CRP/settlement.
2. Mechanical split — **без** зміни CLI/API/схем і навмисної зміни тестів.
3. C2 у CI — **додатковий** job; C0/C1 gate з `#38` не послаблюється.
4. Core ↛ node/peer/CSU; CSU ↛ CSU — firewall `#45` залишається обов’язковим.
5. `OperationalPlane` — reference-local; не production runtime.

## 3. Свідомо не входить (post-F / окреме рішення)

```text
federation leave / Join Request / CRP / settlement ledger
PHM / HIE / Epistemic CSU implementation
production event scheduler / distributed plane
codesign / MSI / App Store
camera QR; non-loopback peer_listen без explicit CLI
```

## 4. Атоми → QUEUE

| ID | Підфаза | Атом | Done when |
|----|---------|------|-----------|
| `#107` | F0 | Toolchain pin | `rust-toolchain.toml` pinned stable version; CI/docs узгоджені |
| `#108` | F0 | Schema manifest gaps | `identifier`, `epistemic-assessment`, `context-artifact` у `fixtures/manifest.json`; CI schema validate зелений |
| `#109` | F0 | CI branch protection doc | `docs/ci-governance.md`: required checks = CI workflow; merge rules для `main` |
| `#110` | F1 | `SECURITY.md` refresh | Політика відповідає фактичному дереву (CSU isolation, Desktop, reporting); не «skeleton only» |
| `#111` | F2 | Handle opacity test | Conformance-тест: handle не розкриває storage path / internal token у attack scenario |
| `#112` | F2 | verify-on-read Object | `ObjectStore::open` / `get_by_object_id` re-verify canonical signature; tamper → fail |
| `#113` | F2 | verify-on-read Artifact | `CasArtifactStore::resolve*` re-verify descriptor signature + content hash |
| `#114` | F2 | Policy Gate dispatch | `CsuRuntime::dispatch` fail-closed без policy або DENY; тести |
| `#115` | F2 | Acquisition fail-closed audit | Regression: download/publish без ALLOW policy → DENY + Evidence; docs |
| `#116` | F3 | Split `model-acquisition` | Mechanical `lib.rs` → modules; тести зелені; поведінка незмінна |
| `#117` | F4 | C2 CI job | GitHub Actions job: `conformance run --profile C2`; червоний при регресії |
| `#118` | F4 | Flaky desktop P2 test | `peer_lifecycle_p2` стабільний на CI (unique bind / isolation) |
| `#119` | F5 | Phase F docs + RFC | `implementation-status.md` + `README.md` sync; RFC-F closure; QUEUE close |

```text
#107 toolchain
  → #108 schema fixtures
    → #109 branch protection doc
      → #110 SECURITY.md
        → #111 Handle opacity
          → #112 verify object
            → #113 verify artifact
              → #114 policy dispatch
                → #115 acquisition audit
                  → #116 split acquisition
                    → #117 C2 CI
                      → #118 flaky P2
                        → #119 docs RFC
```

## 5. Acceptance Phase F

```text
CI: C0+C1 (existing) + C2 job green on main.
Schema validate includes previously PARTIAL schema rows.
SECURITY.md and README reflect E4 DONE + stabilization posture.
Handle opacity + verify-on-read + dispatch policy covered by named tests.
model-acquisition split without behavior change.
No new OPEN stabilization rows without new developer decision.
```

## 6. RFC

- Consolidating docs atom: RFC-F Phase F stabilization (`#119`).
- Per-atom RFC-E лише якщо атом торкається контракту (verify-on-read, dispatch policy).
