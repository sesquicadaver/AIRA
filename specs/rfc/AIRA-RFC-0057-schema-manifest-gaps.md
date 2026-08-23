# AIRA-RFC-0057 — Schema manifest gaps (identifier, epistemic, context)

## 1. Summary

Phase F `#108`: add valid/invalid fixtures + `fixtures/manifest.json` entries for `aira:schema:common:identifier:0.1`, `aira:schema:epistemic:assessment:0.1`, `aira:schema:artifact:context-artifact:0.1`.

## 5. Non-Goals

Schema semantic changes; C2 profile expansion.

## 15. Tests

`cargo run -p aira-cli -- schema validate --fixtures fixtures`
`cargo test -p aira-schema fixture_manifest_passes`
