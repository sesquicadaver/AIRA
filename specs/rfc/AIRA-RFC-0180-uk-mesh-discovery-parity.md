# AIRA-RFC-0180 — UK mesh/discovery label parity (RFC-D)

## 1. Summary

Phase R `#292`: Ukrainian Desktop chrome for mesh status and discovery headings matches EN meaning with Cyrillic user-facing copy. CLI-shaped discovery field labels (`stun_server:`, `discv …`, `peer_listen:`) stay English tech-only by design. RFC-0174 stays file-free until `#294`.

## 2. Problem Statement

UK locale still showed English shells for Identity / Reachability / Direct / Relay / Rendezvous / Discovery and mixed EN loanwords in mesh tech rows, so Connection looked half-translated.

## 3. Motivation

`phase-r-plan` R6: EN==UK gaps for mesh/discovery closed or consciously tech-only.

## 4. Scope

- `crates/aira-desktop/src/app/i18n.rs` UK mesh_* / discovery / peer / profile chrome
- Unit test `uk_mesh_discovery_labels_not_english_shell`
- Living smoke tip `#292` DONE → `#293`; Analyze-328; RFC-0180

## 5. Non-Goals

```text
Cold-start empty-profile guidance (#293)
Consolidating RFC-0174 (#294)
Translating CLI argv / wire enums / help markdown beyond chrome
Rewriting mesh_language mapping
```

## 6. Compatibility / Security

UI strings only. Honesty invariants unchanged (UNKNOWN≠OFFLINE, book≠sessions).

## 7. Rollout

QUEUE `#292` → Analyze-328 → PR; next `#293` cold-start empty profile.
