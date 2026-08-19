# MVP Implementation Roadmap (аналіз)

**Джерело:** `Manifesto etc/AIRA MVP Implementation Roadmap v0.1.md` (1626 рядків)  
**Type:** Reference / Planning  
**Confidence:** High

## Evidence

- **§1–3:** Mission local/minimal/deterministic/…; Non-goals = Book V / later profiles; Target **C0 → C1 → partial C2** (local protocols only).
- **§4–6:** Architecture tree aira-node; stack Rust/tokio/SQLite/CAS/SHA-256/Ed25519/JSON Schema/clap/axum; repo layout crates/csu/schemas/fixtures/tests/specs.
- **§7–20 Milestones M0–M13:** Spec freeze → Schema → Object → Artifact → Event → Policy → Invariants → CSU runtime → Basic CSU → E2E flow → CLI → HTTP → Conformance harness → Partial C2 local protocols.
- **§21 Build order:** specs→schemas→runtimes→CSU→flow→CLI→API→conformance→C2 stubs; **не** починати з network.
- **§23–24:** Security baseline; MUST NOT add Core entities / Node/Driver/Scheduler ontology / GPU/LLM/chain / mutate / hide failures / skip Policy / direct CSU mutation.
- **§25–29 Demos:** 2+2; echo pipeline; failure→evidence; ready reuse; normative split stub.
- **§30 DoD:** 22 пунктів incl. C0/C1 pass.
- **§32–36:** CI pipeline; branching; release `aira-mvp-v0.1.0`; docs; post-MVP C2…C5 path without Core expansion unless RFC-B.
- **§37 Formula:** Book 0–III local + Schema + C0/C1 + demos.

## Inference

Інженерний міст Book IV ↔ Issue Set. M* ≈ Issue epics 0–10.
