# Provenance — Per-CSU Publisher Identity

**Decision:** CSU emits sign via `manifest.publisher_identity` using `signature_for` / `make_*_as`; plane/lifecycle stay on primary.

**Why:** Schema already has publisher_identity; avoids mutating global primary during dispatch.

**Non-goals:** Runtime emit_failed/lifecycle as publisher; node secret rotate; multi-tenant keyrings.
