# Repository Bootstrap Plan (аналіз)

**Джерело:** `Manifesto etc/AIRA Repository Bootstrap Plan v0.1.md` (1295 рядків)  
**Type:** Reference / Planning / Engineering  
**Confidence:** High

## Evidence

- **§1–3:** First repo proves PS→…→Evidence offline; name `aira` monorepo.
- **§4–6:** Root layout (.github, specs, crates×10, csu, schemas, fixtures, tests); crate boundaries aira-core/object/event/artifact/policy/csu/schema/conformance/node/cli.
- **§7–11:** First schemas/fixtures/CLI; local node state; first E2E test.
- **§12–18:** CI + security CI; coding standards; dependency direction; branching; commits; PR template fields.
- **§19 Epics 0–7:** Bootstrap → Spec snapshot → Schema → C0 Core → C1 CSU → Operational flow → CLI/Node → Conformance (згорнутий vs Issue Set 0–11).
- **§20–24:** Alpha release scope; README anti-regression text; demo script; repo invariants; acceptance 12 пунктів (build, CI, specs/, schemas/, fixtures/, crates/, csu/, init, 2+2, C0, C1, no ML/GPU/chain/net).

## Gap (Inference)

Поточний репозиторій `AIRA` містить `Manifesto etc/`, `Meditation_About/`, `Analyze-*` — **ще немає** `crates/`, `schemas/`, `aira` CLI. Bootstrap Acceptance **не виконано**; наступний impl cycle = Issue #1+.
