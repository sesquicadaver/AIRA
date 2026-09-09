# AIRA-RFC-0194 — Opt-in dial off UI-thread (RFC-D)

## 1. Summary

Phase T `#308`: Desktop opt-in peer dial runs on an `AsyncDesktopJobs` dial slot (`try_spawn_dial` / `poll_dial`). The egui thread never `block_on` TCP/Noise. F1 and navigation stay available during an attempt. RFC-0192 stays file-free until `#312`.

## 2. Problem Statement

After `#300` / `#307`, Technical details → Dial still called `run_opt_in_peer_dial` synchronously on the UI path. A slow or unreachable endpoint froze the frame loop (Help/F1 and tab navigation blocked).

## 3. Motivation

`phase-t-plan` `#308` / post-S audit §3: dial via jobs/slot; UI must not `block_on`; F1/nav during attempt.

## 4. Scope

- `AsyncDesktopJobs::{dial_rx`, `try_spawn_dial`, `poll_dial`, `dial_inflight`
- `AiraDesktopApp::request_opt_in_peer_dial` + `pump_async_jobs` apply outcome
- Technical-details Dial button disabled while inflight; waiting label
- Help / `desktop-peer.md` honesty that dial is off-UI
- QUEUE tip → `#309`

## 5. Non-Goals

```text
Observe miss fail durable (#309)
Quit∥Submit completion (#310)
AddressBook candidate honesty / rollback (#311)
Changing dial evidence → live_session_count contract (#307)
Merging dial with submit∥lifecycle exclusivity
```

## 6. Compatibility / Security

Trust + explicit address + evidence persistence unchanged. `block_on` remains inside the dial worker (off egui). No public bind / auto-trust default.

## 7. Rollout

QUEUE `#308` → Analyze-344 → PR; next `#309` Observe miss fail durable.
