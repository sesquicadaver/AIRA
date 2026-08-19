# Provenance — Bootstrap scope

## Що зроблено

- Epic 0 (#1–#5) + Epic 1 snapshot (#6–#7)
- `specs/` = **копії** з `Manifesto etc/` (не move, не edit originals)
- Crates = empty skeletons з smoke `crate_version` tests
- CLI `aira status` — bootstrap only

## Що не зроблено

- Schema JSON files (`schemas/` лише placeholders)
- Object/Event/Artifact/Policy runtime
- Conformance harness
- Issue #8 terminology guardrail

## Гілки

- `main` — stable
- `develop` — integration (створюється в цьому циклі)
