# Phase M — OS child sandbox (post-L)

**Статус:** складено 2026-09-03; **IN PROGRESS** → QUEUE `#224`–`#230` (RFC-0117 reserved).  
**Джерела:** Phase L [`phase-l-plan.md`](phase-l-plan.md) §3.1 Out; RFC-0116: operators who need Landlock / seccomp / netns wait for a post-L atom; QUEUE L closed @ RFC-0111.  
**Канон backlog:** [`QUEUE.md`](../QUEUE.md) секція Phase M `#224`–`#230`.  
**Closure RFC:** `AIRA-RFC-0117` — id **confirmed free** at `#224` (файл лише в `#230`).  
**Не змінює:** анти-місію README (ядро **не** хост інференсу; **не** GPU marketplace); C0/C1 `Calculate 2 + 2`; `aira-core`; AIRA-mediated `network=none` adapter contract (L5 stays honest).

## 0. Навіщо (після L, не замість L)

Phase L зробила межу AIRA → child **чесною**: env whitelist, bounded pipes, activate evidence, ProblemRecord не бреше, `network=none` = adapter opens no sockets.  
Дитина **досі** не OS-ізольована. Це не брехня після RFC-0116, але це дірка для operator-ів, яким потрібен sandbox.

```text
Phase L execution-boundary honesty DONE (#217–#223 / RFC-0111)
  → Phase M: Linux OS isolation for ProcessBackend child
    → не нова LLM функція; не Core inference; не GPU marketplace
```

## 1. Мета

Додати **opt-in** Linux OS sandbox на `ProcessBackend` child: Landlock (FS), seccomp (syscalls), network namespace (net). CI лишає MockBackend. Немає ядра / немає capability → **CapsuleFailed**, не фальшивий VERIFIED. ollama-style loopback лишається явною виняткою; netns цілить offline argv (llama.cpp-style).

Цільовий ярлик: **Reference v0.3-strict + optional Linux OS child sandbox** (не v0.4; не production multi-OS sandbox).

## 2. Інваріанти

1. Лінійність: `#N` після `#N-1`; один атом = один PR.
2. `aira-core` не містить LLM logic / tokenizer / ggml / Landlock.
3. CSU ↛ CSU: sandbox живе в `execution-llm` process path; plane/CLI оркеструє.
4. C1 `Calculate 2 + 2` лишається `execution-basic` + VERIFIED 4.0.
5. Немає backend / немає activate evidence / sandbox required але недоступний → **CapsuleFailed** + Evidence.
6. CI лишає MockBackend; не вимагати ollama/llama.cpp; sandbox тести — `/bin/true` або temp script.
7. Не редагувати `Manifesto etc/`, `Meditation_About/`.
8. AIRA-mediated `network=none` (RFC-0116) не замінюється мовчки на OS `network=none`; документи розрізняють.

## 3. Анти-місія (ніколи в QUEUE як «зробити AIRA LLM runtime»)

```text
GPU marketplace / compute rental
LLM runtime (Core як inference host)
blockchain network / scheduler in Core
Core що «вирішує задачі» замість CSU contracts
```

Phase M **є** дозволеним: OS isolation уже існуючого Execution CSU child. Це не анти-місія.

## 3.1. Поза цим етапом

```text
macOS seatbelt / Windows job objects (post-M)
WASM/hostile sandbox for weights
forcing ollama through netns (loopback exception stays)
cross-platform CI matrix for ProcessBackend/desktop
LLM result semantic verification / remote execution
full Book II wire / distributed OperationalPlane
model marketplace
inventing Phase N
```

## 4. Смуги (одна лінія)

```text
M0 govern (wiring)
  → M1 Landlock FS
    → M2 seccomp syscalls
      → M3 Linux network namespace
        → M4 missing-kernel / non-Linux fail-closed
          → M5 docs/schema OS vs AIRA-mediated
            → M6 consolidating RFC-0117
```

## 5. Атоми → QUEUE `#224`–`#230`

| ID | Підфаза | Атом | Done when | Не в цьому атомі |
|----|---------|------|-----------|------------------|
| `#224` | M0 | Phase M wiring **DONE** | цей план **IN PROGRESS**; QUEUE `#224` DONE | Landlock (#225) |
| `#225` | M1 | Landlock FS **DONE** | Linux Landlock on ProcessBackend child; FS restriction fail-closed | seccomp (#226) |
| `#226` | M2 | seccomp **DONE** | syscall filter on child; forbidden syscall → CapsuleFailed | netns (#227) |
| `#227` | M3 | netns **DONE** | Linux network namespace for offline argv; ollama loopback not silently broken | policy (#228) |
| `#228` | M4 | missing sandbox fail-closed | sandbox required + no kernel/OS → CapsuleFailed, not VERIFIED | schema (#229) |
| `#229` | M5 | OS vs AIRA-mediated docs | schema/docs distinguish OS isolation from RFC-0116 adapter none | RFC-0117 (#230) |
| `#230` | M6 | RFC-0117 + close | consolidating RFC-0117; QUEUE M closed | — |

```text
#224 M0
  → #225 M1 Landlock
    → #226 M2 seccomp
      → #227 M3 netns
        → #228 M4 fail-closed policy
          → #229 M5 docs
            → #230 M6 close
```

## 6. Acceptance Phase M

```text
QUEUE `#224`–`#230` DONE; no OPEN M atoms.
C1 2+2 still VERIFIED 4.0 via execution-basic.
Linux ProcessBackend child can be OS-isolated (Landlock + seccomp + netns).
Missing kernel / non-Linux + sandbox required → CapsuleFailed.
RFC-0116 AIRA-mediated none remains an adapter contract; OS isolation is explicit.
CI keeps MockBackend; no ollama required.
aira-core has no inference.
Анти-місія (marketplace / Core-host) не порушена.
```

## 7. RFC

- Consolidating (`#230`): `AIRA-RFC-0117` — id **confirmed free** at `#224`.
- Per-atom RFC-S/RFC-D у `#225`–`#229` (окремі файли; не RFC-A; не RFC-0117 до close).

## 8. Activation rule

```text
#223 DONE (RFC-0111) + phase-l §3.1 Out + developer «наступний пункт» 2026-09-03 → цей план активовано атомом `#224`.
RFC-0117 file-free until `#230`.
```
