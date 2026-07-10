# Conformance Test Specification (аналіз)

**Джерело:** `Manifesto etc/AIRA Conformance Test Specification v0.1.md` (1039 рядків)  
**Type:** Normative / Conformance  
**Confidence:** High

## Evidence

- **§1–2:** Перевіряє сумісність (не якість ML/routing/PHM/UI). Basis = Book 0–III.
- **§4:** Profiles AIRA-C0…C5 (узгоджено з SCD §10); C5 optional.
- **§5–6:** Conformance Artifact (immutable); results PASS|FAIL|SKIP|UNSUPPORTED|INVALID.
- **§7 B0:** Ontology recognition; forbidden Core (GPU/LLM/Node/Driver/Scheduler/Blockchain/Wallet/HIE/Goal/KG/Execution Plan); pipeline shape; Human Final Collapse; Evidence primacy.
- **§8 B1:** Immutability, handle opacity, causality, no global total order, Policy ALLOW|DENY|REQUIRE, invariants, security boundary, VRA completeness.
- **§9 B2:** Envelope/signature/version; Discovery by Capability; CRP; Artifact resolve; Event idempotency; Federation autonomy; Settlement privacy.
- **§10–11 B3 + roles:** Manifest; no direct CSU call; Failure Event; role tests CTX/RED/EVD/EPI/EXE/VER/ART/DIS/SET/PHM/HUM.
- **§12–16:** OP pipeline; SEP Operational/Evolution; SEC; PRIV; FAIL suites.
- **§18–22:** Determinism; fixtures; report; compatibility decision; Minimal Pass Matrix C0…C5.
- **§23:** `aira-conformance run --profile C1 --target …` (CLI naming vs Book IV `aira conformance` — TF7-007).

## Inference

Нормативний **oracle** сумісності. MVP target = C0+C1 (+ partial local C2 у Roadmap). Role tests включають PHM/HUM — узгоджено з Book III §37 (15), не з §12 (13).
