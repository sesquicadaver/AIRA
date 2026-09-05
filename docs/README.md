# AIRA documentation index

**Оновлено:** 2026-09-05. Канон виконання: [`QUEUE.md`](../QUEUE.md) (Phase N `#231`–`#236` **DONE**; first OPEN `#237`; Phase M `#224`–`#230` **DONE** @ RFC-0117; QUEUE M closed; Phase L `#217`–`#223` **DONE** @ RFC-0111; QUEUE L closed; Phase K `#209`–`#216` **DONE** @ RFC-0104; QUEUE K closed; Phase J `#199`–`#208` **DONE** @ RFC-0096; QUEUE J closed; Phase I `#184`–`#198` **DONE** @ RFC-0078; QUEUE I closed; Phase H `#152`–`#183` **DONE** @ RFC-0077; Phase G `#120`–`#151` **DONE**). Статус реалізації: [`implementation-status.md`](implementation-status.md) (**Reference v0.3-strict**; [`phase-n-plan.md`](phase-n-plan.md); [`phase-m-plan.md`](phase-m-plan.md); [`phase-l-plan.md`](phase-l-plan.md); [`AIRA-RFC-0117`](../specs/rfc/AIRA-RFC-0117-phase-m-os-child-sandbox.md); [`AIRA-RFC-0111`](../specs/rfc/AIRA-RFC-0111-phase-l-execution-boundary.md); [`AIRA-RFC-0104`](../specs/rfc/AIRA-RFC-0104-phase-k-local-llm-csu.md); [`AIRA-RFC-0096`](../specs/rfc/AIRA-RFC-0096-phase-j-book-gap-local.md); [`AIRA-RFC-0078`](../specs/rfc/AIRA-RFC-0078-phase-i-semantic-stabilization.md); [`AIRA-RFC-0077`](../specs/rfc/AIRA-RFC-0077-phase-h-protocol-depth-v0.3.md)).

## Anti-mission (ніколи в product / QUEUE)

README §«What AIRA is not» — **не** backlog:

```text
GPU marketplace · LLM runtime · blockchain network · scheduler
Core що «вирішує задачі» замість CSU contracts
```

Деталі: [`phase-g-plan.md`](phase-g-plan.md) §3; [`canonical-terminology.md`](canonical-terminology.md).

## Phase plans (лінійна черга)

| Phase | Plan | QUEUE | Статус |
|-------|------|-------|--------|
| C | [`phase-c-plan.md`](phase-c-plan.md) | `#38`–`#52` | **DONE** |
| D | [`phase-d-plan.md`](phase-d-plan.md) | `#53`–`#74` | **DONE** |
| E | [`phase-e-plan.md`](phase-e-plan.md) + [`desktop-ux.md`](desktop-ux.md) | `#75`–`#106` | **DONE** |
| F | [`phase-f-plan.md`](phase-f-plan.md) | `#107`–`#119` | **DONE** |
| G | [`phase-g-plan.md`](phase-g-plan.md) | `#120`–`#151` | **DONE** (RFC-0069) |
| H | [`phase-h-plan.md`](phase-h-plan.md) | `#152`–`#183` | **DONE** (RFC-0077) |
| I | [`phase-i-plan.md`](phase-i-plan.md) | `#184`–`#198` | **DONE** (`#184` `#185` `#186` `#187` `#188` `#189` `#190` `#191` `#192` `#193` `#194` `#195` `#196` `#197` `#198` @ RFC-0078) |
| J | [`phase-j-plan.md`](phase-j-plan.md) | `#199`–`#208` | **DONE** (RFC-0096) |
| K | [`phase-k-plan.md`](phase-k-plan.md) | `#209`–`#216` | **DONE** (RFC-0104); QUEUE K closed |
| L | [`phase-l-plan.md`](phase-l-plan.md) | `#217`–`#223` | **DONE** @ RFC-0111; QUEUE L closed |
| M | [`phase-m-plan.md`](phase-m-plan.md) | `#224`–`#230` | **DONE** @ RFC-0117; QUEUE M closed |
| N | [`phase-n-plan.md`](phase-n-plan.md) | `#231`–`#247` | **IN PROGRESS** (`#231`–`#236` DONE @ RFC-0124/0125/0126/0127/0128; first OPEN `#237`; RFC-0123 reserved) |

## Core reference

| Doc | Topic |
|-----|-------|
| [`implementation-status.md`](implementation-status.md) | Book 0–IV matrix; C0–C5; Phase gates; **Reference v0.3-strict**; QUEUE J closed; QUEUE K closed @ RFC-0104; QUEUE L closed @ RFC-0111; QUEUE M closed @ RFC-0117; Phase N IN PROGRESS |
| [`rfc-p-promotion.md`](rfc-p-promotion.md) | RFC-P: Book V → operational лише через promotion (`#177`) |
| [`operational-plane.md`](operational-plane.md) | C1 reference/demo plane (не production runtime) |
| [`conformance.md`](conformance.md) | C0–C5 runners; v0.3 index; security baseline |
| [`ci-governance.md`](ci-governance.md) | GitHub Actions merge gate |
| [`demo.md`](demo.md) | Local 2+2 and failure paths |
| [`canonical-terminology.md`](canonical-terminology.md) | Forbidden Core vocabulary |

## CSU & models

| Doc | Topic |
|-----|-------|
| [`csu-development.md`](csu-development.md) | Manifest, trait, basic set |
| [`model-acquisition-policy.md`](model-acquisition-policy.md) | Download/publish fail-closed |
| [`csu-sdk.md`](csu-sdk.md) | Third-party CSU contract (`#145`; not marketplace) |

## Local node & network (post-MVP)

| Doc | Topic |
|-----|-------|
| [`local-node.md`](local-node.md) | `.aira` layout, HTTP, identity; generate-local `network=none` honesty (RFC-0116) |
| [`crypto.md`](crypto.md) | Ed25519, Noise, TLS, keyring |
| [`peer-link.md`](peer-link.md) | Peer listen, trust, relay, DHT prototypes |
| [`runbook-systemd.md`](runbook-systemd.md) | Long-running daemons |

## Desktop (Developer Preview)

| Doc | Topic |
|-----|-------|
| [`desktop-ux.md`](desktop-ux.md) | Product decisions P0–P6 |
| [`desktop-network-profiles.md`](desktop-network-profiles.md) | P0–P6 fail-closed rules |
| [`desktop-gui.md`](desktop-gui.md) | `aira-desktop` GUI |
| [`desktop-launcher.md`](desktop-launcher.md) | Linux `.desktop` launcher |
| [`desktop-peer.md`](desktop-peer.md) | Supervised peer lifecycle |
| [`desktop-invite.md`](desktop-invite.md) | PeerInvite file + QR PNG |
| [`desktop-federation.md`](desktop-federation.md) | P5 federation GUI |
| [`desktop-discovery.md`](desktop-discovery.md) | P6 STUN/discv (Dev) |
| [`desktop-packaging.md`](desktop-packaging.md) | Linux tarball + `.deb` pipeline (`#150`) |
| [`desktop-packaging-macos.md`](desktop-packaging-macos.md) | macOS `.app` tarball + codesign/notarize helper (`#148`) |
| [`desktop-packaging-windows.md`](desktop-packaging-windows.md) | Windows zip + MSI pipeline (`#149`) |

## Releases & security

| Doc | Topic |
|-----|-------|
| [`RELEASE_NOTES-v0.1.0-alpha.1.md`](RELEASE_NOTES-v0.1.0-alpha.1.md) | Alpha release notes |
| [`../SECURITY.md`](../SECURITY.md) | Reporting, controls, out-of-scope |

## Historical / provenance

| Doc | Topic |
|-----|-------|
| [`../NEXT_PROBLEM.md`](../NEXT_PROBLEM.md) | Desktop one-click problem (RESOLVED → Phase E) |
| [`../EVO-1.md`](../EVO-1.md) | Repo analysis snapshot |
| [`../EVO-2.md`](../EVO-2.md) | Evolution recommendations |
