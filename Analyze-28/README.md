# Analyze-28 — Dual-Key Grace Window

**Scope:** Optional verify grace for rotated-out peers until explicit `--until` RFC3339 UTC.

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. Grace is opt-in via `--until`; omit = immediate cutover (Analyze-27)
2. During grace: old + new both verify; old remains on CRL (no re-upsert)
3. After `until`: sync unloads old verifying key
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** `RevokedEntry.grace_until` + `to_keyring_at(now)` / sync includes active grace pubkeys; CLI `--until`
- **Why:** Explicit operator-controlled dual-key window without fixed TTL policy
- **Alternatives:** Fixed 24h TTL (rejected by operator); dual-key always-on (unsafe)
- **Follow-ups:** per-CSU publisher; node signing-secret rotate

### Acceptance
- rotate without until → old fails verify (file ring)
- rotate with future until → old+new OK at now; past now → only new
- upsert(old) still RevokedKey during grace
- workspace tests + clippy PASS
