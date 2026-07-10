# Schema Pack (аналіз)

**Джерело:** `Manifesto etc/AIRA Schema Pack v0.1.md` (1841 рядків)  
**Type:** Supporting / Data contracts  
**Confidence:** High

## Evidence

- **§1–3:** JSON Schema 2020-12; canonical serialization; schema IDs `aira:schema:…:0.1`; scalars Identifier/Ref/Hash/Timestamp/Signature/Scope.
- **§4–26:** Descriptors: Core Object, PS, Context, Artifact, Event, Protocol Envelope/Response, Identity, Policy Query/Decision, Capability, CSU Manifest, Capsule, Evidence, Epistemic, VRA, Differentiated Field, Human Choice, Failure payload, Research, Promotion Candidate, Settlement Receipt, Conformance Report.
- **§27 Profile requirements:** C0+ core set; C1+ CSU/Capability/Capsule/Evidence/Epistemic; C2+ protocol/identity; C3+ federation/CRP; C4+ settlement; C5+ research/promotion/human choice.
- **§28 Forbidden couplings:** gpu_id, node_id as core identity, driver_id, scheduler_id, wallet mandatory, blockchain_height mandatory, llm_model_id core, global_state_version.
- **§29–31:** Extensions `x-<org>-…`; registry list; minimal impl must validate PS/Context/Capsule/Event/Artifact/CSU/Capability/Policy/VRA/Conformance Report.

## Inference

Машинний шар між Books і harness. Не входить у Standard formula SCD §15 напряму, але Conformance/MVP без нього неавтоматизовані. Next doc у Status = RFC Process.
