//! Prime Private Port Invariant + deterministic selection (QUEUE #232–#233 / Phase N).
//!
//! AIRA-owned TCP/UDP peer transport endpoints MUST use a prime port in
//! `49152..=65535` (`P_AIRA`, exactly 1491 values). This is a cheap structural
//! pre-filter — not authentication. Outbound to Polygon RPC / STUN / HTTP is
//! out of scope (not AIRA-owned transport).
//!
//! `#233`: `preferred_port(identity, transport_class)` hashes
//! `identity_ref || class || version` (SHA-256) into `P_AIRA`; collisions walk
//! the next primes with wrap, never spinning forever.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
use std::sync::LazyLock;

use aira_object::ContentHash;

use crate::error::PeerError;

/// Domain tag mixed into preferred-port selection (`aira-prime` §6).
pub const PORT_SELECT_VERSION: &str = "aira:port-select:v1";

/// Transport classes that own an AIRA listen/advertise port (not HTTP/STUN/RPC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportClass {
    /// Authenticated peer TCP (`listen` / address book).
    TcpPeer,
    /// UDP discv announce / FIND.
    UdpDiscv,
}

impl TransportClass {
    /// Stable ASCII token concatenated into the selection hash.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TcpPeer => "tcp-peer",
            Self::UdpDiscv => "udp-discv",
        }
    }
}

/// Inclusive floor of the Dynamic/Private range scanned for `P_AIRA`.
pub const P_AIRA_RANGE_MIN: u16 = 49152;
/// Inclusive ceiling of the Dynamic/Private range scanned for `P_AIRA`.
pub const P_AIRA_RANGE_MAX: u16 = 65535;
/// Smallest prime in `P_AIRA`.
pub const P_AIRA_FIRST: u16 = 49157;
/// Largest prime in `P_AIRA`.
pub const P_AIRA_LAST: u16 = 65521;
/// Exact cardinality of `P_AIRA`.
pub const P_AIRA_COUNT: usize = 1491;

static P_AIRA: LazyLock<Vec<u16>> = LazyLock::new(build_p_aira);

fn is_prime_u16(n: u16) -> bool {
    if n < 2 {
        return false;
    }
    if n.is_multiple_of(2) {
        return n == 2;
    }
    let mut d = 3u32;
    let n32 = u32::from(n);
    while d * d <= n32 {
        if n32.is_multiple_of(d) {
            return false;
        }
        d += 2;
    }
    true
}

fn build_p_aira() -> Vec<u16> {
    let ports: Vec<u16> = (P_AIRA_RANGE_MIN..=P_AIRA_RANGE_MAX)
        .filter(|&p| is_prime_u16(p))
        .collect();
    debug_assert_eq!(ports.len(), P_AIRA_COUNT);
    debug_assert_eq!(ports.first().copied(), Some(P_AIRA_FIRST));
    debug_assert_eq!(ports.last().copied(), Some(P_AIRA_LAST));
    ports
}

/// Sorted view of all ports in `P_AIRA`.
pub fn p_aira_ports() -> &'static [u16] {
    &P_AIRA
}

/// True iff `port` is an element of `P_AIRA`.
pub fn is_prime_port(port: u16) -> bool {
    p_aira_ports().binary_search(&port).is_ok()
}

/// Same as [`is_prime_port`] — valid AIRA-owned transport port.
pub fn is_valid_aira_port(port: u16) -> bool {
    is_prime_port(port)
}

/// Fail-closed port check with operator-facing diagnostics.
pub fn validate_aira_port(port: u16) -> Result<(), PeerError> {
    if is_valid_aira_port(port) {
        return Ok(());
    }
    Err(PeerError::InvalidPort(format!(
        "Configured AIRA peer port {port} is invalid under Prime Private Port Invariant. \
         Select a prime port in {P_AIRA_RANGE_MIN}–{P_AIRA_RANGE_MAX}. \
         Suggested port: {P_AIRA_FIRST}"
    )))
}

/// Parse `host:port` (or bare socket) and return the port component.
pub fn parse_bind_port(bind: &str) -> Result<u16, PeerError> {
    let addr: SocketAddr = bind.parse().map_err(|e| {
        PeerError::InvalidPort(format!(
            "invalid AIRA bind `{bind}`: {e}; expected host:port with a prime port in \
             {P_AIRA_RANGE_MIN}–{P_AIRA_RANGE_MAX} (suggested {P_AIRA_FIRST})"
        ))
    })?;
    Ok(addr.port())
}

/// Fail-closed: bind string must parse and use a `P_AIRA` port.
pub fn validate_aira_bind(bind: &str) -> Result<u16, PeerError> {
    let port = parse_bind_port(bind)?;
    validate_aira_port(port)?;
    Ok(port)
}

/// First free TCP prime on loopback (no identity hashing — that is `#233`).
pub fn select_available_loopback_tcp() -> Result<(TcpListener, SocketAddr), PeerError> {
    for &port in p_aira_ports() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        match TcpListener::bind(addr) {
            Ok(listener) => {
                let local = listener.local_addr().map_err(PeerError::from)?;
                return Ok((listener, local));
            }
            Err(_) => continue,
        }
    }
    Err(PeerError::InvalidPort(
        "no free AIRA prime TCP port available on 127.0.0.1".into(),
    ))
}

/// First free UDP prime on loopback (discv tests / operators).
pub fn select_available_loopback_udp() -> Result<(UdpSocket, SocketAddr), PeerError> {
    for &port in p_aira_ports() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        match UdpSocket::bind(addr) {
            Ok(sock) => {
                let local = sock.local_addr().map_err(PeerError::from)?;
                return Ok((sock, local));
            }
            Err(_) => continue,
        }
    }
    Err(PeerError::InvalidPort(
        "no free AIRA prime UDP port available on 127.0.0.1".into(),
    ))
}

/// Format `127.0.0.1:<available-prime>` for Desktop/CLI tests.
pub fn format_available_loopback_tcp_bind() -> Result<String, PeerError> {
    let (listener, addr) = select_available_loopback_tcp()?;
    drop(listener);
    Ok(addr.to_string())
}

/// SHA-256 input bytes: `identity_ref || transport_class || version` (TZ §6).
fn selection_preimage(identity_ref: &str, class: TransportClass) -> Vec<u8> {
    let mut buf =
        Vec::with_capacity(identity_ref.len() + class.as_str().len() + PORT_SELECT_VERSION.len());
    buf.extend_from_slice(identity_ref.as_bytes());
    buf.extend_from_slice(class.as_str().as_bytes());
    buf.extend_from_slice(PORT_SELECT_VERSION.as_bytes());
    buf
}

/// Index into `P_AIRA` for `identity_ref` + `class` (`0..P_AIRA_COUNT`).
pub fn preferred_port_index(identity_ref: &str, class: TransportClass) -> usize {
    let digest = ContentHash::sha256_bytes(&selection_preimage(identity_ref, class));
    let hex = digest
        .as_str()
        .strip_prefix("sha256:")
        .expect("ContentHash::sha256_bytes always prefixes sha256:");
    let bytes = hex::decode(hex).expect("sha256 hex is valid");
    debug_assert_eq!(bytes.len(), 32);
    let mut word = [0u8; 8];
    word.copy_from_slice(&bytes[..8]);
    let n = u64::from_be_bytes(word);
    (n % (P_AIRA_COUNT as u64)) as usize
}

/// Deterministic preferred AIRA port for an identity and transport class.
pub fn preferred_port(identity_ref: &str, class: TransportClass) -> u16 {
    p_aira_ports()[preferred_port_index(identity_ref, class)]
}

/// Alias used in operator diagnostics (`suggested preferred`).
pub fn suggested_aira_port(identity_ref: &str, class: TransportClass) -> u16 {
    preferred_port(identity_ref, class)
}

/// Port at `preferred_index + offset` (mod `|P_AIRA|`).
pub fn next_candidate_port_from_index(preferred_index: usize, offset: usize) -> u16 {
    let idx = preferred_index
        .wrapping_add(offset)
        .rem_euclid(P_AIRA_COUNT);
    p_aira_ports()[idx]
}

/// Next candidate after `current` in the wrap-around walk that started at `preferred`.
///
/// `current` must be in `P_AIRA`; otherwise returns [`PeerError::InvalidPort`].
pub fn next_candidate_port(preferred: u16, current: u16) -> Result<u16, PeerError> {
    let ports = p_aira_ports();
    let start = ports.binary_search(&preferred).map_err(|_| {
        PeerError::InvalidPort(format!(
            "preferred port {preferred} is not in P_AIRA (suggested {P_AIRA_FIRST})"
        ))
    })?;
    let cur = ports.binary_search(&current).map_err(|_| {
        PeerError::InvalidPort(format!(
            "current port {current} is not in P_AIRA (suggested {P_AIRA_FIRST})"
        ))
    })?;
    let offset = cur.wrapping_sub(start).rem_euclid(P_AIRA_COUNT) + 1;
    Ok(next_candidate_port_from_index(start, offset))
}

/// Walk `P_AIRA` from the preferred index until `is_free` accepts a port (finite wrap).
pub fn select_available_port<F>(
    identity_ref: &str,
    class: TransportClass,
    mut is_free: F,
) -> Result<u16, PeerError>
where
    F: FnMut(u16) -> bool,
{
    let start = preferred_port_index(identity_ref, class);
    for offset in 0..P_AIRA_COUNT {
        let port = next_candidate_port_from_index(start, offset);
        if is_free(port) {
            return Ok(port);
        }
    }
    Err(PeerError::InvalidPort(format!(
        "no free AIRA prime port available after full P_AIRA walk \
         (preferred {} for {identity_ref} / {})",
        preferred_port(identity_ref, class),
        class.as_str()
    )))
}

/// Bind loopback TCP starting at the identity's preferred prime (collision → next).
pub fn select_available_loopback_tcp_for(
    identity_ref: &str,
) -> Result<(TcpListener, SocketAddr), PeerError> {
    let start = preferred_port_index(identity_ref, TransportClass::TcpPeer);
    for offset in 0..P_AIRA_COUNT {
        let port = next_candidate_port_from_index(start, offset);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        match TcpListener::bind(addr) {
            Ok(listener) => {
                let local = listener.local_addr().map_err(PeerError::from)?;
                return Ok((listener, local));
            }
            Err(_) => continue,
        }
    }
    Err(PeerError::InvalidPort(format!(
        "no free AIRA prime TCP port on 127.0.0.1 after full walk \
         (preferred {})",
        preferred_port(identity_ref, TransportClass::TcpPeer)
    )))
}

/// Bind loopback UDP starting at the identity's preferred discv prime.
pub fn select_available_loopback_udp_for(
    identity_ref: &str,
) -> Result<(UdpSocket, SocketAddr), PeerError> {
    let start = preferred_port_index(identity_ref, TransportClass::UdpDiscv);
    for offset in 0..P_AIRA_COUNT {
        let port = next_candidate_port_from_index(start, offset);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        match UdpSocket::bind(addr) {
            Ok(sock) => {
                let local = sock.local_addr().map_err(PeerError::from)?;
                return Ok((sock, local));
            }
            Err(_) => continue,
        }
    }
    Err(PeerError::InvalidPort(format!(
        "no free AIRA prime UDP port on 127.0.0.1 after full walk \
         (preferred {})",
        preferred_port(identity_ref, TransportClass::UdpDiscv)
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn p_aira_cardinality_and_bounds() {
        let ports = p_aira_ports();
        assert_eq!(ports.len(), P_AIRA_COUNT);
        assert_eq!(ports[0], P_AIRA_FIRST);
        assert_eq!(*ports.last().unwrap(), P_AIRA_LAST);
        assert!(ports.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn accepts_first_and_last_prime() {
        assert!(is_prime_port(49157));
        assert!(is_prime_port(65521));
        validate_aira_port(49157).unwrap();
        validate_aira_port(65521).unwrap();
    }

    #[test]
    fn rejects_range_edges_and_service_ports() {
        for port in [
            49152u16, 65535, 443, 80, 22, 53, 123, 993, 3306, 5432, 8080, 0, 9797,
        ] {
            assert!(!is_valid_aira_port(port), "port {port} must be rejected");
            assert!(validate_aira_port(port).is_err());
        }
    }

    #[test]
    fn rejects_composite_private_port() {
        // 50000 = even composite inside Dynamic/Private range
        assert!(!is_prime_port(50000));
        let err = validate_aira_port(50000).unwrap_err().to_string();
        assert!(err.contains("Prime Private Port Invariant"));
        assert!(err.contains("49157"));
    }

    #[test]
    fn validate_bind_ok_and_fail() {
        assert_eq!(validate_aira_bind("127.0.0.1:49157").unwrap(), 49157);
        assert!(validate_aira_bind("127.0.0.1:0").is_err());
        assert!(validate_aira_bind("127.0.0.1:443").is_err());
        assert!(validate_aira_bind("127.0.0.1:9797").is_err());
    }

    #[test]
    fn same_identity_same_preferred_port() {
        let id = "aira:identity:port-select-alice";
        let a = preferred_port(id, TransportClass::TcpPeer);
        let b = preferred_port(id, TransportClass::TcpPeer);
        assert_eq!(a, b);
        assert!(is_valid_aira_port(a));
        assert_eq!(suggested_aira_port(id, TransportClass::TcpPeer), a);
        // Different class may differ (not required to differ, but index is independent).
        let _udp = preferred_port(id, TransportClass::UdpDiscv);
        assert!(is_valid_aira_port(_udp));
    }

    #[test]
    fn different_identities_usually_differ() {
        let a = preferred_port("aira:identity:ps-a", TransportClass::TcpPeer);
        let b = preferred_port("aira:identity:ps-b", TransportClass::TcpPeer);
        // Collisions allowed statistically; just ensure both valid.
        assert!(is_valid_aira_port(a));
        assert!(is_valid_aira_port(b));
    }

    #[test]
    fn collision_walks_next_prime() {
        let id = "aira:identity:collision-walk";
        let preferred = preferred_port(id, TransportClass::TcpPeer);
        let mut blocked = HashSet::new();
        blocked.insert(preferred);
        let next =
            select_available_port(id, TransportClass::TcpPeer, |p| !blocked.contains(&p)).unwrap();
        assert_ne!(next, preferred);
        assert_eq!(next, next_candidate_port(preferred, preferred).unwrap());
    }

    #[test]
    fn full_wrap_is_finite_and_errors() {
        let id = "aira:identity:full-wrap";
        let err = select_available_port(id, TransportClass::TcpPeer, |_| false).unwrap_err();
        assert!(err.to_string().contains("full P_AIRA walk"), "{err}");
    }

    #[test]
    fn next_candidate_wraps_from_last_index() {
        let last = P_AIRA_COUNT - 1;
        let first_again = next_candidate_port_from_index(last, 1);
        assert_eq!(first_again, P_AIRA_FIRST);
        let preferred = p_aira_ports()[last];
        assert_eq!(
            next_candidate_port(preferred, preferred).unwrap(),
            P_AIRA_FIRST
        );
    }

    #[test]
    fn selection_index_in_range() {
        for name in ["a", "b", "aira:identity:z", ""] {
            let idx = preferred_port_index(name, TransportClass::TcpPeer);
            assert!(idx < P_AIRA_COUNT);
        }
    }
}
