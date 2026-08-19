# RFC Template & Change Process (аналіз)

**Джерело:** `Manifesto etc/AIRA RFC Template & Change Process v0.1.md` (1364 рядків)  
**Type:** Normative / Governance  
**Confidence:** High

## Evidence

- **§1–3:** Контрольована еволюція; захист від GPU/Node/Driver/Scheduler/Blockchain-first regression; basis = Book 0–III + Conformance + Schema.
- **§4 Classes (primary MUST one):** A Architecture, B Core/ABI, C Protocol, D CSU, **S Schema**, **T Conformance Test**, E Reference, R Research, **P Promotion**, **X Deprecation**.
- **§5–14:** Admission criteria per class; RFC-P evidence + promotion gate; RFC-X supersession.
- **§15–18:** Metadata + document template (20 sections); lifecycle proposed→…→validated/superseded.
- **§20–31:** Compatibility impact model; versioning major/minor/patch; gates Architecture/Core/Protocol/CSU/Schema/Research Promotion.
- **§32 Decisions:** ACCEPT | ACCEPT_WITH_CHANGES | REJECT | DEFER | SEND_TO_RESEARCH | REQUIRE_MORE_EVIDENCE.
- **§36–39:** Fast-track / Emergency; rejection criteria; active corpus protection.
- **§44 Status next:** MVP Implementation Roadmap.

## Tension

SCD §8 lists лише A/B/C/D/E/R. RFC Template додає S/T/P/X — **канон для process = RFC Template** (новіший/повніший); SCD потребує RFC-X sync — TF7-002.
