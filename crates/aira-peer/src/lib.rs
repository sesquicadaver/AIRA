//! AIRA authenticated peer links (Analyze-32…59).
//!
//! Framed TCP + mutual Ed25519 hello v1 + Noise XX + signed [`ProtocolEnvelope`].
//! Admission is local [`TrustStore`] only — no controlling center.
//! Trust-delta (`peer.trust.delta`) can propagate CRL ops and same-id rekey over the encrypted link.
//! Analyze-43: optional gossip fanout + durable `peers/discovery.json`.
//! Analyze-44: relay-first hub (`peer.relay.deliver` + address-book `via`).
//! Analyze-47: trusted-mesh DHT-lite (`peers/dht.json` + `peer.dht.announce`).
//! Analyze-59: `accept_tcp` + `complete_accept` so daemon loops are not blocked by handshake.
//! Analyze-66: STUN Binding client + `peers/stun_reflexive.json` (no ICE; dial stays TCP/book).
//! Analyze-67: UDP discv5-style announce (signed datagram → local DHT store).
//! Analyze-68: iterative FIND_NODE over UDP discv (XOR closest; no apply-book).
//! QUEUE #232: Prime Private Port Invariant for AIRA-owned peer/discv/relay endpoints.
//! QUEUE #234: Node Presence Record (canonical Ed25519).
//! QUEUE #235: RendezvousProvider trait + mock (no ledger in Core).
//! QUEUE #236: EvmRendezvousProvider local double + Amoy/mainnet config hooks.
//! QUEUE #237: publish/query product layer (TTL/sequence) + rendezvous.json.
//! QUEUE #238: peer-assisted Reachability Probe (no hairpin proof).
//! QUEUE #239: Reachability states + peers/reachability.json.
//! QUEUE #240: Presence → AddressBook promotion (trust-gated).
//! QUEUE #241: Relay integration (direct→NAT→relay; prime ads; dual reservation).
//! QUEUE #242: Presence refresh (sequence++; expire stale; endpoint change).
//! QUEUE #245: Ab ovo publish → discover → trust → dial (no preconfigured peers).

mod ab_ovo;
mod address_book;
mod dht;
mod discovery;
mod discv;
mod envelope;
mod error;
mod evm_rendezvous;
mod frame;
mod gossip;
mod handshake;
mod noise;
mod notify;
mod presence;
mod presence_promote;
mod presence_refresh;
mod prime_port;
mod reachability;
mod reachability_state;
mod relay;
mod relay_integrate;
mod rendezvous;
mod rendezvous_ops;
mod replay;
mod session;
mod stun;
mod trust_delta;

pub use ab_ovo::{admit_peer_trust, discover_admit_promote, record_discovered_presence};
pub use address_book::{AddressBook, PeerEndpoint};
pub use dht::{
    apply_book_exact_from_dht_find, apply_dht_announce, apply_dht_announce_maybe_book,
    dht_announce_to_peers, dht_key_hex, make_dht_announce_envelope, parse_dht_announce,
    promote_dht_to_address_book, xor_distance, DhtAnnounce, DhtRecord, PeerDhtStore,
    DHT_ANNOUNCE_MESSAGE_TYPE, DHT_DEFAULT_K, DHT_SCHEMA,
};
pub use discovery::{DiscoveryEntry, DiscoverySource, PeerDiscoveryStore};
pub use discv::{
    apply_discv_announce, apply_discv_datagram, bind_udp, bind_udp_explicit, handle_discv_datagram,
    iterative_discv_find, recv_one_and_handle, recv_one_and_store, send_discv_announce,
    set_udp_timeout, sign_discv_announce, DiscvAnnounce, DiscvFindReport, DiscvHandleResult,
    DISCV_ANNOUNCE_DOMAIN, DISCV_FIND_ALPHA, DISCV_FIND_MAX_HOPS, DISCV_FIND_SCHEMA,
    DISCV_NODES_SCHEMA, DISCV_SCHEMA,
};
pub use envelope::make_peer_ping;
pub use error::PeerError;
pub use evm_rendezvous::{
    evm_identity_hash, EvmChainProfile, EvmRendezvousConfig, EvmRendezvousProvider,
    EVM_AMOY_RPC_DEFAULT, EVM_CHAIN_AMOY, EVM_CHAIN_LOCAL_DOUBLE, EVM_CHAIN_POLYGON,
    EVM_LOCAL_CONTRACT_PLACEHOLDER, EVM_POLYGON_RPC_DEFAULT, RENDEZVOUS_KIND_EVM,
};
pub use frame::{read_frame, write_frame, MAX_FRAME_BYTES};
pub use gossip::{
    gossip_forward_trust_delta, gossip_mark_seen, GossipForwardResult, GossipSeenLog,
    GOSSIP_SEEN_CAP,
};
pub use handshake::{HelloMessage, HelloResult, HELLO_DOMAIN};
pub use noise::{
    list_noise_static_backups, load_or_create_noise_static, prune_noise_static_backups,
    rotate_noise_static, x25519_public, NoiseStaticBackupInfo, NoiseStaticPruneReport,
    NoiseStaticRotate, NODE_X25519_BACKUP_FILE, NOISE_PATTERN,
};
pub use notify::{
    notify_peer_of_rekey, notify_peers_of_rekey, upcoming_rekey_delta, NotifyPeerResult,
};
pub use presence::{
    empty_capabilities_hash, presence_now, presence_to_value, NodePresenceRecord,
    PresenceDirectEndpoint, PresenceDraft, PresenceReachability, PresenceRelayEndpoint,
    PRESENCE_SCHEMA, PUBLIC_NETWORK_ID,
};
pub use presence_promote::{
    dial_target_from_presence, promote_presence_to_address_book, trust_policy_allows,
};
pub use presence_refresh::{
    endpoint_change_and_sign_presence, endpoint_change_presence_draft, is_presence_expired,
    presence_endpoints_equal, refresh_and_sign_presence, refresh_presence_draft,
    retain_unexpired_presence, sign_presence_draft_for_node, trusted_peers_to_notify,
    PRESENCE_REFRESH_TTL_SECS_DEFAULT,
};
pub use prime_port::{
    format_available_loopback_tcp_bind, is_prime_port, is_valid_aira_port, next_candidate_port,
    next_candidate_port_from_index, p_aira_ports, parse_bind_port, preferred_port,
    preferred_port_index, select_available_loopback_tcp, select_available_loopback_tcp_for,
    select_available_loopback_udp, select_available_loopback_udp_for, select_available_port,
    suggested_aira_port, validate_aira_bind, validate_aira_port, TransportClass,
    PORT_SELECT_VERSION, P_AIRA_COUNT, P_AIRA_FIRST, P_AIRA_LAST, P_AIRA_RANGE_MAX,
    P_AIRA_RANGE_MIN,
};
pub use reachability::{
    ChallengeDraft, ReachabilityAttestation, ReachabilityChallenge, ReachabilityReplayLog,
    ReachabilityResult, REACHABILITY_ATTESTATION_SCHEMA, REACHABILITY_CHALLENGE_SCHEMA,
    REACHABILITY_REPLAY_CAP, REACHABILITY_RESULT_SCHEMA,
};
pub use reachability_state::{
    ReachabilityLocalState, ReachabilityStatus, RelayRouteRecord, REACHABILITY_STATE_SCHEMA,
};
pub use relay::{
    make_relay_deliver_envelope, parse_relay_deliver, send_envelope_to_peer, serve_relay_peer,
    with_relay_hub_registry, RelayDeliver, RelayHub, RelayHubEntry, RelayHubRegistry,
    RELAY_DELIVER_MESSAGE_TYPE, RELAY_DELIVER_SCHEMA, RELAY_HUB_REGISTRY_SCHEMA,
    RELAY_HUB_TTL_DAYS_RECOMMENDED,
};
pub use relay_integrate::{
    nat_observed_from_root, plan_dial_path, plan_dial_path_with_relays, select_relay_reservations,
    DialPathInput, DialPathStep, RelayAdDraft, RelayAdStore, RelayAdvertisement,
    RELAY_ADS_STATE_SCHEMA, RELAY_ADVERTISEMENT_SCHEMA, RELAY_AD_NETWORK_ID,
    RELAY_RESERVATION_TARGET,
};
pub use rendezvous::{
    LocalFileRendezvousProvider, MockRendezvousProvider, RendezvousProvider,
    RENDEZVOUS_KIND_LOCAL_FILE, RENDEZVOUS_KIND_MOCK, RENDEZVOUS_LEDGER_SCHEMA,
};
pub use rendezvous_ops::{
    encode_evm_publish_call, presence_ttl_secs, EvmPublishCall, RendezvousClient,
    RendezvousLocalState, RendezvousPublishPolicy, RENDEZVOUS_MAX_QUERY_RESULTS,
    RENDEZVOUS_MAX_RECORD_BYTES, RENDEZVOUS_MAX_TTL_SECS, RENDEZVOUS_MIN_TTL_SECS,
    RENDEZVOUS_STATE_SCHEMA,
};
pub use replay::{admit_received_envelope, envelope_replay_path};
pub use session::{
    accept, accept_tcp, complete_accept, dial, listen, listen_available_loopback, listen_explicit,
    AuthenticatedPeer, DEFAULT_PEER_TIMEOUT,
};
pub use stun::{
    build_binding_request, parse_binding_success, query_and_save_stun_reflexive,
    query_stun_reflexive, resolve_dht_announce_addr, MockStunServer, StunReflexiveRecord,
    STUN_MAGIC_COOKIE, STUN_QUERY_TIMEOUT,
};
pub use trust_delta::{
    apply_trust_delta, local_rekey_delta, make_trust_delta_envelope, parse_trust_delta, TrustDelta,
    TrustDeltaOp, TRUST_DELTA_MESSAGE_TYPE, TRUST_DELTA_SCHEMA,
};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod smoke_tests;
