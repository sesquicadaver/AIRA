# Provenance — Conformance C0/C1 scope

## Sources
- `Manifesto etc/AIRA Initial Issue Set v0.1.md` Epic 9 (#63–#70)
- `schemas/conformance/report.schema.json`
- Existing Epic 4 InvariantChecker / PolicyGate; Epic 7 OperationalPlane

## Mapping
| Issue | Module |
|-------|--------|
| #63 | `report.rs` + `runner::finalize_suite` |
| #64/#66–#69 | `c0.rs` |
| #65/#70 | `c1.rs` |
| CLI | `aira conformance run --profile C0|C1` |
