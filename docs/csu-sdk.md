# CSU SDK contract (third-party)

**Phase G `#145`.** Contract for loading an external CSU **manifest** into a local node registry. This is **not** a CSU marketplace, SDK binary distribution, or remote plugin host.

See also: [`csu-development.md`](csu-development.md) (in-tree basic CSUs), [`conformance.md`](conformance.md), fixture `fixtures/valid/csu/manifest-external-partner.json` (`#144`).

## Scope

| In | Out |
|----|-----|
| Signed `CsuManifest` JSON (`aira:schema:csu:manifest:0.1`) | Publishing / discovering CSUs on a network |
| Local `csu register` / `CsuRegistry` | Shipping `.so` / WASM plugins from Core |
| Schema + canonical signature verify | Marketplace UI or billing |
| Named conformance case `c1.csu.external_partner_fixture` | Full third-party execution sandbox productization |

## Contract checklist

1. **Identity** — `csu_id` is an `aira:csu:…` ref **not** required to match a crate under `csu/` (external partner ids are allowed).
2. **ABI** — `abi_version` must be `0.1` (`SUPPORTED_ABI_VERSION`).
3. **Signature** — non-empty `signature.signature_value`; `verify_canonical` / `validate_for_registration` must pass.
4. **Publisher** — `publisher_identity` / `signature.key_ref` must verify under the process trust / local-test fixture keys used for Alpha signing.
5. **Sandbox** — declare fail-closed defaults for basic partners (`network: none`, `secret_access: none`, `filesystem: none` unless the partner documents a narrower need).
6. **Events** — `event_subscriptions` / `event_outputs` are declarative; registration does **not** auto-wire a production dispatcher (C1 `OperationalPlane` remains reference/demo).

## Validate & register

```bash
# Schema (fixtures index includes the partner file)
cargo run -p aira-cli -- schema validate --fixtures fixtures

# Single file
cargo run -p aira-cli -- schema validate \
  --schema aira:schema:csu:manifest:0.1 \
  --file fixtures/valid/csu/manifest-external-partner.json

# Local registry
cargo run -p aira-cli -- --root /tmp/aira-sdk-demo init
cargo run -p aira-cli -- --root /tmp/aira-sdk-demo csu register \
  --manifest fixtures/valid/csu/manifest-external-partner.json --activate
cargo run -p aira-cli -- --root /tmp/aira-sdk-demo csu list
```

## Conformance

| Case id | Profile | Checks |
|---------|---------|--------|
| `c1.csu.external_partner_fixture` | C1 | Load partner fixture → schema validate → `validate_for_registration` → `CsuRegistry::register` + save/load |

Run:

```bash
cargo run -p aira-cli -- conformance run --profile C1 --out /tmp/aira-c1
```

## Anti-mission

AIRA is **not** a GPU marketplace, LLM runtime host, blockchain network, or scheduler. External CSUs do not change Core ontology; they remain contracts + local registration. README §«What AIRA is not».
