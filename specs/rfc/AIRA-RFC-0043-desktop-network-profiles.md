# AIRA-RFC-0043 — Desktop network profiles documentation

## 1. Summary

Consolidating documentation atom (`#106`): `docs/desktop-network-profiles.md` maps P0–P6 profiles to supervised peer flags, settings fields, GUI surfaces, QUEUE atoms `#94`–`#105`, and per-profile RFC-E index. Cross-links `desktop-ux`, `desktop-peer`, `peer-link`, `desktop-gui`, federation and discovery docs.

## 5. Non-Goals

Peer protocol changes; new runtime behavior; packaging updates.

## 7. Change

- `docs/desktop-network-profiles.md`
- Cross-links added in `desktop-ux.md`, `desktop-peer.md`, `peer-link.md`, `phase-e-plan.md`, `implementation-status.md`
- Tests: doc contract in `network_profiles_doc.rs`

## 15. Tests

`cargo test -p aira-desktop-runtime --test network_profiles_doc`
