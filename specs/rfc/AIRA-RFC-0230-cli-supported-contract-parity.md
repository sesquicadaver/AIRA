# AIRA-RFC-0230 — CLI supported-contract parity (RFC-D)

## 1. Summary

QUEUE `#347` (Phase X / Pack 2 M4): `aira problem submit` exposes only admission fields that are **enforced** or **explicit fail-closed**. HTTP/CLI share allowed + excluded model sets. Unsupported knobs are not offered as CLI flags.

## 2. Problem Statement

RFC-0229 froze `excluded_model_refs` for HTTP/Desktop JSON, but CLI could not express excludes. Legacy `--temperature`, `--privacy-class`, and `--allow-*-fallback` looked like generation/policy controls while RFC-0218 always rejected them — a false supported contract (no-op surface).

## 3. Motivation

Phase X [`docs/phase-x-plan.md`](../../docs/phase-x-plan.md) M4; parent RFC-0226 reserved until `#358`.

## 4. Scope

- `--excluded-model-ref` (repeatable) → `AdmissionConstraints.excluded_model_refs`
- Keep: `--model-ref`, `--allowed-model-ref`, `--placement`, `--reuse-policy`
- Remove CLI flags: `--temperature`, `--privacy-class`, `--allow-model-fallback`, `--allow-placement-fallback`
- Clap rejects removed flags; Auto-within-set freeze unchanged (`#346`)

## 5. Non-Goals

```text
Settings Models catalog GUI (#348)
Work executor / Compare
Adding unsupported knobs (top_p, budget, model_version) as flags
Weakening activate evidence/hash admit
Consolidating RFC-0226 (#358)
```

## 6. Compatibility

HTTP still accepts full `AdmissionConstraints` JSON (deny_unknown; unsupported → enforce_or_reject). Removing CLI flags is a **breaking CLI surface** for anyone who passed temperature/privacy/fallback — those paths already failed closed at admit. `--placement remote_required` remains and still fails closed.

## 7. Acceptance

```text
--allowed-model-ref A,B --excluded-model-ref A → snapshot.model_ref = B
omit --excluded-model-ref → excluded_model_refs empty
--temperature / --privacy-class / --allow-*-fallback → clap reject
Tip → first OPEN #348
```

## 8. References

- QUEUE `#347` · Analyze-384
- Parent: RFC-0226; prior RFC-0229 profile snapshot; RFC-0218 enforce-or-reject; RFC-0210 submit constraints
