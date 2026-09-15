# AIRA-RFC-0235 — Human copy + Settings≠System IA (RFC-D)

## 1. Summary

QUEUE `#352` (Phase X / Pack 2 X2): Program buttons name the object of the action; Settings owns network edit; System Connection observes + invites + one CTA.

## 2. Problem Statement

Stop/Refresh/Quit did not say what they act on. Network profile edit lived on System → Connection while Settings only mirrored Saved≠Applied — operators had two competing “settings” surfaces.

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) X2 after network honesty (#351). Parent RFC-0226 reserved until `#358`.

## 4. Scope

- Labels: Start node / Stop node / Refresh status / Exit AIRA (EN) and UK equivalents
- Related CTA, restart hints, problem_action, lexicon, Help lifecycle/apply/connect copy
- Settings → Connection: edit P0–P2, peer listen, P3|P4 radio
- System → Connection: observe profile + invites + open-Settings jump; no second profile form
- strip `no model selected` (no raw `none selected`)

## 5. Non-Goals

```text
Help F1 model path (#353)
Compare mode (#354)
Changing lifecycle semantics / quit-during-submit (#310)
Weakening connection CTA logic
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

Lifecycle Start/Stop/Refresh/Quit behavior unchanged. Enable-private-network CTA may still apply P1 as one corrective action on System.

## 7. Acceptance

```text
Program buttons: Stop node / Refresh status / Exit AIRA (and UK)
Settings Connection edits profile/listen; System does not duplicate the form
Help node.lifecycle / settings.apply name the new buttons
Tip → first OPEN #353
```

## 8. References

- QUEUE `#352` · Analyze-389
- Parent: RFC-0226; prior RFC-0234 address honesty; desktop-ux IA
