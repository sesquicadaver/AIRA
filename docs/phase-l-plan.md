# Phase L — Execution-boundary hardening (post-K)

**Статус:** складено 2026-09-03; **IN PROGRESS** → QUEUE `#217`–`#223` (RFC-0111 reserved).  
**Джерела:** post-K audit (`aira-1.md` / 2026-09-03): Core semantic contracts стабільні; ризик на межі AIRA → child LLM; [`phase-k-plan.md`](phase-k-plan.md) QUEUE K closed @ RFC-0104.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase L `#217`–`#223`.  
**Closure RFC:** `AIRA-RFC-0111` — id **confirmed free** at `#217` (файл лише в `#223`).  
**Не змінює:** анти-місію README (ядро **не** хост інференсу; **не** GPU marketplace); C0/C1 `Calculate 2 + 2`; `aira-core`; Landlock/seccomp/network-namespace sandbox (Out L).

## 0. Навіщо (після K, не замість K)

Phase K дала `execution-llm` + MockBackend + ProcessBackend + pointer activate gate.  
Аудит: `network: none` не OS-гарантія; child успадковує env; activate = JSON presence; `ProblemRecord` мітить executed як verified; pipes unbounded.

```text
Phase K Local LLM CSU DONE (#209–#216 / RFC-0104)
  → Phase L: harden AIRA → external local LLM process boundary
    → не нова LLM функція; не Core inference
```

## 1. Мета

Зробити Phase K execution boundary **fail-closed і чесним**: activate = Phase D evidence/hash, не pointer-exists; child env whitelist; bounded stdout/stderr; disk ProblemRecord не бреше про VERIFIED; `network: none` = AIRA-mediated (adapter не відкриває socket), не фальшивий OS-sandbox.

Цільовий ярлик: **Reference v0.3-strict + hardened local LLM CSU** (не v0.4; не production sandbox).

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` не містить LLM logic / tokenizer / ggml.
3. CSU ↛ CSU: `execution-llm` не залежить Cargo від `model-inventory` / `model-acquisition`; plane/CLI оркеструє.
4. C1 `Calculate 2 + 2` лишається `execution-basic` + VERIFIED 4.0.
5. Немає backend / немає activate evidence → **CapsuleFailed** + Evidence, не фальшивий VERIFIED.
6. CI лишає MockBackend; не вимагати ollama/llama.cpp.
7. Не редагувати `Manifesto etc/`, `Meditation_About/`.

## 3. Анти-місія (ніколи в QUEUE як «зробити AIRA LLM runtime»)

```text
GPU marketplace / compute rental
LLM runtime (Core як inference host)
blockchain network / scheduler in Core
Core що «вирішує задачі» замість CSU contracts
```

Phase L **є** дозволеним: hardening уже існуючого Execution CSU. Це не анти-місія.

## 3.1. Поза цим етапом

```text
Linux network namespace / Landlock / seccomp for child (post-L)
cross-platform CI matrix for ProcessBackend/desktop
LLM result semantic verification / remote execution
full Book II wire / distributed OperationalPlane
model marketplace
WASM/hostile sandbox for weights
```

`#222` **locks honest contract**: `constraints.network = none` means the AIRA adapter opens no sockets. The child process is **not** an OS-isolated network-off sandbox. ollama-style loopback remains an explicit host-process exception, documented, not claimed as `network=none` OS enforcement.

## 4. Смуги (одна лінія)

```text
L0 govern (wiring)
  → L1 activate evidence/hash (not pointer-exists)
    → L2 child env whitelist
      → L3 bounded stdout/stderr
        → L4 ProblemRecord execution_artifact_id
          → L5 network=none honest contract RFC
            → L6 consolidating RFC-0111
```

## 5. Атоми → QUEUE `#217`–`#223`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#217` | L0 | Phase L wiring | цей план **IN PROGRESS**; QUEUE `#217` DONE + `#218`–`#223` OPEN; `phase_l_doc`; `NEXT_PROBLEM.md` drift fixed | activate evidence (#218) |
| `#218` | L1 | Activate evidence | gate checks `content_hash` + cache bytes + evidence artifact; forged `{model_ref}` fail-closed **DONE** (RFC-0112) | env (#219) |
| `#219` | L2 | Child env whitelist | `env_clear` + minimal PATH/HOME/LANG; no `AIRA_HTTP_TOKEN` leak | bounded IO (#220) |
| `#220` | L3 | Bounded pipes | stdout/stderr capped **during** read; overflow → CapsuleFailed | ProblemRecord (#221) |
| `#221` | L4 | ProblemRecord split | persisted `execution_artifact_id`; GET не кладе executed в `verified_artifact_id` | network RFC (#222) |
| `#222` | L5 | `network=none` contract | RFC: AIRA-mediated none; not OS sandbox; schema/docs honest | RFC-0111 body (#223) |
| `#223` | L6 | RFC-0111 + close | consolidating RFC-0111; QUEUE L closed | — |

```text
#217 L0
  → #218 L1 activate evidence
    → #219 L2 env
      → #220 L3 bounded IO
        → #221 L4 ProblemRecord
          → #222 L5 network contract
            → #223 L6 close
```

## 6. Acceptance Phase L

```text
QUEUE `#217`–`#223` DONE; no OPEN L atoms.
C1 2+2 still VERIFIED 4.0 via execution-basic.
Activate is evidence/hash, not pointer-exists.
Child env is a whitelist; pipes are bounded.
ProblemRecord does not label executed as verified.
network=none is an honest AIRA-adapter contract (not OS sandbox).
aira-core has no inference.
Анти-місія (marketplace / Core-host) не порушена.
```

## 7. RFC

- Consolidating (`#223`): `AIRA-RFC-0111` — id **confirmed free** at `#217`.
- Per-atom RFC-S/RFC-D у `#218`–`#222` (окремі файли; не RFC-A; не RFC-0111 до close).

## 8. Activation rule

```text
#216 DONE (RFC-0104) + post-K audit 2026-09-03 + /autopilot → цей план активовано атомом `#217`.
#217 wiring DONE (RFC-0112 path); `#218` Activate evidence DONE; `#219`–`#223` OPEN.
RFC-0111 file-free until `#223`.
```
