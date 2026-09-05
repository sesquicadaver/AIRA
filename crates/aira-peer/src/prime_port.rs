//! Prime Private Port Invariant (QUEUE #232 / Phase N).
//!
//! AIRA-owned TCP/UDP peer transport endpoints MUST use a prime port in
//! `49152..=65535` (`P_AIRA`, exactly 1491 values). This is a cheap structural
//! pre-filter — not authentication. Outbound to Polygon RPC / STUN / HTTP is
//! out of scope (not AIRA-owned transport).
//!
//! Deterministic `preferred_port(identity, …)` lands in QUEUE `#233`.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
use std::sync::LazyLock;

use crate::error::PeerError;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
