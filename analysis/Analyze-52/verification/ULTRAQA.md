# UltraQA — Analyze-52

## Goal
Self-sovereign trust-delta: subject≠issuer rejected; self-ops / gossip / relay apply; send gate rejects foreign subject.

## Scenario matrix

| ID | Intent | Result | Evidence |
|----|--------|--------|----------|
| U1 | Third-party revoke/unrevoke/rotate/rekey | IdentityMismatch | `trust_delta_ops_require_issuer_subject_match` ok |
| U2 | Self-revoke over Noise peer | applies on peer | `trust_delta_revoke_roundtrip_applies` ok |
| U3 | Gossip self-announce A→B→C | CRL on B+C | `gossip_trust_delta_a_to_b_to_c` ok |
| U4 | Relay self-announce A→C via R | CRL on C | `relay_hub_delivers_trust_delta_a_to_c_via_r` ok |
| U5 | Protected local-test/local self | Protocol | `trust_delta_refuses_local_test_and_local_node` ok |
| U6 | Stranger rotate old→new | IdentityMismatch; self-rotate ok | `trust_delta_rotate_shape_and_apply` ok |
| U7 | Send envelope foreign subject | IdentityMismatch | `make_trust_delta_envelope_rejects_foreign_subject` |
| U8 | Full suite + clippy | green | 31 tests; clippy -D warnings |

## Verdict
**PASS** — adversarial scenarios green; no product defects.

## Cleanup
No temporary harness left in tree.
