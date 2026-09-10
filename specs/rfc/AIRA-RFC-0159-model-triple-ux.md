# AIRA-RFC-0159 — Model triple UX (selected ≠ ready ≠ used) (RFC-D)

## 1. Summary

Phase P `#269` (historical): Desktop surfaces three independent model facts — **selected** (Phase D `activated.latest.json` `model_ref`), **ready** (full activate evidence/hash confirmation), **used-in-result** (from the last Work payload only). Strip, System, and Settings observe the triple.

**Contract amendment (2026-09-10):** Settings **MUST** provide user-facing **local model selection / activation** among discovered compatible models (anti-usurpation of a single LLM). Observe-only / “not an inventory editor” as the Settings Models contract is **superseded**. The triple **selected ≠ ready ≠ used** remains normative. Choosing a model does **not** imply VERIFIED and must not fake VERIFIED.

## 2. Problem Statement

### 2.1 Original (`#269`)

Strip always showed «not checked»; System ignored `view.model`; Settings treated a placeholder as the Models control. Selected could be confused with used.

### 2.2 Amendment driver (2026-09-10)

Observe-only Settings blocked the user from choosing which local LLM Capability to bind. That effect conflicts with anti-usurpation (no single LLM owns the node) and with Book 0 composition of **available capabilities**. Fail-closed VERIFIED remains; denial of **choice** is not required for honesty.

## 3. Motivation

`phase-p-plan` and post-O audit: selected ≠ prepared ≠ used in the result.

Amendment: local model choice in Desktop is a primary product surface (EVO-3 / Phase D model contour), not a CLI-only workaround. GPU marketplace / compute rental stay anti-mission.

## 4. Scope

### 4.1 Delivered in `#269`

- `ActivatedPointerGate::observe` / `ActivationObservation`
- `ModelTripleSnapshot` + strip/System/Settings UI (observation of the triple)
- `WorkResultView.used_model` extraction
- Tests; QUEUE → `#270`

### 4.2 Normative after amendment (implementation may be a later QUEUE atom)

- Settings → Models: user can **list** local discovered/compatible models and **select/activate** one (writes the same Phase D activate path as `aira models activate`)
- Strip / System continue to show selected ≠ ready ≠ used honestly after selection
- WorkSubmit / generate failures when no model is activated MUST NOT be primarily framed as «AIRA is not running» when the node lifecycle is Running
- Domain rating / share surfaces remain as already documented in Phase D (D5–D7); this amendment does not reopen remote model marketplace

## 5. Non-Goals

```text
Reachability endpoint bind (#270) — historical
GPU marketplace / compute rental / paid placement (anti-mission)
Remote model registry / DHT model market / silent remote fetch of foreign models
Consolidating RFC-0156 (#274) — historical
Claiming selection ⇒ VERIFIED or ready ⇒ process LLM without evidence
Rewriting Core as LLM runtime
```

**Superseded Non-Goal (do not cite as current contract):**

```text
Full inventory editor   — superseded for *local* select/activate in Settings
Settings observe-only   — superseded 2026-09-10
```

## 6. Compatibility / Security

No Core/ledger changes. Ready never means VERIFIED. C1 `Calculate 2 + 2` still needs no model. Activate remains fail-closed (Phase D / Execution CSU). `share_custom_models` default DENY unchanged.

## 7. Rollout

QUEUE `#269` → Analyze-304 → PR (triple observation) — **DONE**.

Amendment 2026-09-10: contract text in this RFC + UX/canon tips; **UI select/activate delivery** = separate linear QUEUE atom(s) after an activated phase plan (do not invent OPEN rows in this docs-only revision unless the developer activates a phase).

## 8. Status honesty

```text
selected ≠ ready ≠ used     = still required
Settings observe-only       = superseded
Local UI select/activate    = required by contract; may lag implementation
GPU marketplace             = still forbidden
VERIFIED honesty            = unchanged (choice ≠ verified result)
```
