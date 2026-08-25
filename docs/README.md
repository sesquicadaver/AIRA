# AIRA documentation index

**Оновлено:** 2026-08-26. Канон виконання: [`QUEUE.md`](../QUEUE.md) (перший OPEN = `#130`). Статус реалізації: [`implementation-status.md`](implementation-status.md).

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
| G | [`phase-g-plan.md`](phase-g-plan.md) | `#120`–`#146` | **OPEN** |

## Core reference

| Doc | Topic |
|-----|-------|
| [`implementation-status.md`](implementation-status.md) | Book 0–IV matrix; C0–C2; Phase gates |
| [`operational-plane.md`](operational-plane.md) | C1 reference/demo plane (не production runtime) |
| [`conformance.md`](conformance.md) | C0/C1/C2 runners; security baseline |
| [`ci-governance.md`](ci-governance.md) | GitHub Actions merge gate |
| [`demo.md`](demo.md) | Local 2+2 and failure paths |
| [`canonical-terminology.md`](canonical-terminology.md) | Forbidden Core vocabulary |

## CSU & models

| Doc | Topic |
|-----|-------|
| [`csu-development.md`](csu-development.md) | Manifest, trait, basic set |
| [`model-acquisition-policy.md`](model-acquisition-policy.md) | Download/publish fail-closed |
| `csu-sdk.md` | Planned Phase G `#140` (not yet in tree) |

## Local node & network (post-MVP)

| Doc | Topic |
|-----|-------|
| [`local-node.md`](local-node.md) | `.aira` layout, HTTP, identity |
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
| [`desktop-packaging.md`](desktop-packaging.md) | Linux tarball |
| [`desktop-packaging-macos.md`](desktop-packaging-macos.md) | macOS `.app` tarball |
| [`desktop-packaging-windows.md`](desktop-packaging-windows.md) | Windows zip |

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
