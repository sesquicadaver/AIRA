# Desktop network profiles P0–P6 (QUEUE #106)

Canonical map of Desktop **network profiles** from [`desktop-ux.md`](desktop-ux.md) §3 through Addendum E4 (`#94`–`#105`). Desktop extends settings + supervise + GUI; peer protocol semantics live in [`peer-link.md`](peer-link.md).

**Posture:** Developer Preview — not production distributed AIRA.

## Profile matrix

| Profile | HTTP node | Supervised `peer listen` | Settings extras | GUI surface | QUEUE / RFC |
|---------|-----------|--------------------------|-----------------|-------------|-------------|
| **P0** | always loopback | none | `peer_listen=null` | default profile | E1 `#75`–`#79`; RFC-0024 |
| **P1** | always | `--recv` | `peer_listen` (default `127.0.0.1:9797`) | profile selector; invite file/QR | E1.1 `#80`–`#85`; [`desktop-peer.md`](desktop-peer.md) |
| **P2** | always | `--recv --dht --apply-book` | same as P1 | peer status (DHT mode) | `#94`–`#96`; RFC-0044–0046 |
| **P3** | always | `--relay --relay-ttl-days N` (no `--recv`) | `relay_ttl_days` (default **31**) | Advanced relay toggle + TTL | `#97`–`#99`; RFC-0047–0049 |
| **P4** | always | `--recv --dht --apply-book --apply-trust --gossip` | mutex with P3 relay flags | Advanced gossip toggle | `#100`–`#102`; RFC-0050–0052 |
| **P5** | always | peer flags per P0–P4 profile | federation membership file | Federation import wizard | `#103`–`#104`; RFC-0053–0054; [`desktop-federation.md`](desktop-federation.md) |
| **P6** | always | (no new supervise flags) | — | Dev panel: STUN / discv / FIND | `#105`; RFC-0055; [`desktop-discovery.md`](desktop-discovery.md) |

## Fail-closed rules (all profiles)

1. Runtime accepts only profiles ≤ current DONE level; higher profiles rejected in settings normalize (fail-closed).
2. No `--allow-public-bind`, no public STUN default, no auto-trust strangers, no hidden port auto-increment.
3. `peer_listen` loopback-only in Desktop until explicit peer CLI wiring (see [`desktop-peer.md`](desktop-peer.md)).
4. **Mutex P3 | P4:** settings cannot select relay profile and gossip profile simultaneously; one supervised `peer listen` line.
5. P6 discovery shortcuts do **not** update TrustStore or AddressBook.

## Apply order

After changing `network_profile`, `peer_listen`, or P3 `relay_ttl_days`: **Stop → Start** in GUI (or `aira desktop stop` / `start`) to respawn supervised peer.

## Cross-links

| Topic | Doc |
|-------|-----|
| UX canon | [`desktop-ux.md`](desktop-ux.md) |
| Plan / acceptance E4 | [`phase-e-plan.md`](phase-e-plan.md) §4d |
| Peer supervise detail | [`desktop-peer.md`](desktop-peer.md) |
| Invite / QR | [`desktop-invite.md`](desktop-invite.md) |
| GUI controls | [`desktop-gui.md`](desktop-gui.md) |
| Federation join | [`desktop-federation.md`](desktop-federation.md) |
| P6 discovery ops | [`desktop-discovery.md`](desktop-discovery.md) |
| Peer protocol CLI | [`peer-link.md`](peer-link.md) |
| Consolidating RFC | [`AIRA-RFC-0043`](../specs/rfc/AIRA-RFC-0043-desktop-network-profiles.md) |

## Per-profile RFC index (E4)

| RFC | Scope |
|-----|--------|
| RFC-0044 | Settings P2 |
| RFC-0045 | Lifecycle P2 (`--dht --apply-book`) |
| RFC-0046 | GUI P2 |
| RFC-0047 | Settings P3 + `relay_ttl_days` |
| RFC-0048 | Lifecycle P3 relay |
| RFC-0049 | GUI P3 Advanced |
| RFC-0050 | Settings P4 gossip |
| RFC-0051 | Lifecycle P4 gossip |
| RFC-0052 | GUI P4 Advanced |
| RFC-0053 | Federation join library |
| RFC-0054 | GUI P5 federation |
| RFC-0055 | P6 Advanced discovery |
