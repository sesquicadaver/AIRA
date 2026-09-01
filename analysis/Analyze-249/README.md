# Analyze-249 — Activate gate (QUEUE #214)

## Done
- `execution-llm` requires an injected `ModelActivateGate`; no gate / `NeverActivated` → `CapsuleFailed` (not VERIFIED)
- `AlwaysActivated` + `MockBackend` still `CapsuleCompleted` (`mock_backend_completes_valid_generate_local`)
- Plane default fail-closed; tests use `enable_activated_mock_llm`; `LocalSession` binds `ActivatedPointerGate` (`models/activated.latest.json`)
- C1 `Calculate 2 + 2` unchanged (`math.eval.safe` / execution-basic)
- RFC-D [`AIRA-RFC-0109`](../../specs/rfc/AIRA-RFC-0109-activate-gate.md)
- QUEUE `#214` **DONE**; first OPEN `#215`
- RFC-0104 remains file-free (`phase_k_rfc_0104_id_free`)

## Out
Process backend (`#215`); Desktop Work generate; RFC-0104 (`#216`); Core inference host; Reduction catalog change; CSU→CSU Cargo dep on inventory/acquisition; real model download / inventory mutation.
