//! STUN Binding client + reflexive addr persistence (Analyze-66 / QUEUE #31).
//!
//! RFC 5389 Binding request/response with XOR-MAPPED-ADDRESS. No ICE, no TURN.
//! Dial path remains TCP from the address book; this module only discovers a
//! reflexive `IP:port` for operators / `dht announce --from-stun`.

use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::PeerError;

/// RFC 5389 magic cookie.
pub const STUN_MAGIC_COOKIE: u32 = 0x2112_A442;
/// Binding request method/class.
const MSG_BINDING_REQUEST: u16 = 0x0001;
/// Binding success response.
const MSG_BINDING_SUCCESS: u16 = 0x0101;
/// XOR-MAPPED-ADDRESS attribute.
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;
/// Legacy MAPPED-ADDRESS (optional fallback).
const ATTR_MAPPED_ADDRESS: u16 = 0x0001;

const IPV4_FAMILY: u8 = 0x01;

/// Default Binding query timeout (single attempt window; retries share this budget).
pub const STUN_QUERY_TIMEOUT: Duration = Duration::from_secs(3);
const STUN_MAX_ATTEMPTS: u32 = 3;

/// Durable observed reflexive address (`peers/stun_reflexive.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StunReflexiveRecord {
    pub addr: String,
    pub stun_server: String,
    /// Unix UTC seconds when observed.
    pub observed_at: u64,
}

impl StunReflexiveRecord {
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("peers").join("stun_reflexive.json")
    }

    pub fn load(root: impl AsRef<Path>) -> Result<Self, PeerError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Err(PeerError::Stun(format!(
                "missing {} — run `peer stun query` first",
                path.display()
            )));
        }
        let raw = fs::read_to_string(&path).map_err(|e| PeerError::Stun(e.to_string()))?;
        serde_json::from_str(&raw).map_err(|e| PeerError::Stun(e.to_string()))
    }

    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), PeerError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| PeerError::Stun(e.to_string()))?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|e| PeerError::Stun(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| PeerError::Stun(e.to_string()))
    }
}

/// Build a 20-byte Binding request with a fresh 96-bit transaction id.
pub fn build_binding_request(txid: &[u8; 12]) -> [u8; 20] {
    let mut msg = [0u8; 20];
    msg[0..2].copy_from_slice(&MSG_BINDING_REQUEST.to_be_bytes());
    msg[2..4].copy_from_slice(&0u16.to_be_bytes()); // length
    msg[4..8].copy_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
    msg[8..20].copy_from_slice(txid);
    msg
}

/// Parse Binding success and extract mapped address (prefers XOR-MAPPED-ADDRESS).
pub fn parse_binding_success(buf: &[u8], expect_txid: &[u8; 12]) -> Result<SocketAddr, PeerError> {
    if buf.len() < 20 {
        return Err(PeerError::Stun("truncated STUN header".into()));
    }
    let msg_type = u16::from_be_bytes([buf[0], buf[1]]);
    if msg_type != MSG_BINDING_SUCCESS {
        return Err(PeerError::Stun(format!(
            "unexpected STUN type {msg_type:#06x} (want Binding success)"
        )));
    }
    let magic = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    if magic != STUN_MAGIC_COOKIE {
        return Err(PeerError::Stun("bad STUN magic cookie".into()));
    }
    let mut txid = [0u8; 12];
    txid.copy_from_slice(&buf[8..20]);
    if &txid != expect_txid {
        return Err(PeerError::Stun("STUN transaction id mismatch".into()));
    }
    let declared_len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
    if buf.len() < 20 + declared_len {
        return Err(PeerError::Stun("truncated STUN attributes".into()));
    }
    let attrs = &buf[20..20 + declared_len];
    let mut xor_addr = None;
    let mut mapped = None;
    let mut i = 0;
    while i + 4 <= attrs.len() {
        let atype = u16::from_be_bytes([attrs[i], attrs[i + 1]]);
        let alen = u16::from_be_bytes([attrs[i + 2], attrs[i + 3]]) as usize;
        i += 4;
        if i + alen > attrs.len() {
            return Err(PeerError::Stun("attribute overruns message".into()));
        }
        let value = &attrs[i..i + alen];
        match atype {
            ATTR_XOR_MAPPED_ADDRESS => {
                xor_addr = Some(decode_xor_mapped(value, &txid)?);
            }
            ATTR_MAPPED_ADDRESS => {
                mapped = Some(decode_mapped(value)?);
            }
            _ => {}
        }
        // 32-bit padding
        i += alen;
        let pad = (4 - (alen % 4)) % 4;
        i += pad;
    }
    xor_addr
        .or(mapped)
        .ok_or_else(|| PeerError::Stun("no XOR-MAPPED-ADDRESS / MAPPED-ADDRESS".into()))
}

fn decode_mapped(value: &[u8]) -> Result<SocketAddr, PeerError> {
    if value.len() < 8 {
        return Err(PeerError::Stun("MAPPED-ADDRESS too short".into()));
    }
    let family = value[1];
    let port = u16::from_be_bytes([value[2], value[3]]);
    if family != IPV4_FAMILY {
        return Err(PeerError::Stun(
            "only IPv4 MAPPED-ADDRESS supported in this slice".into(),
        ));
    }
    if value.len() < 8 {
        return Err(PeerError::Stun("MAPPED-ADDRESS IPv4 truncated".into()));
    }
    let ip = Ipv4Addr::new(value[4], value[5], value[6], value[7]);
    Ok(SocketAddr::new(IpAddr::V4(ip), port))
}

fn decode_xor_mapped(value: &[u8], txid: &[u8; 12]) -> Result<SocketAddr, PeerError> {
    if value.len() < 8 {
        return Err(PeerError::Stun("XOR-MAPPED-ADDRESS too short".into()));
    }
    let family = value[1];
    if family != IPV4_FAMILY {
        return Err(PeerError::Stun(
            "only IPv4 XOR-MAPPED-ADDRESS supported in this slice".into(),
        ));
    }
    let xport = u16::from_be_bytes([value[2], value[3]]);
    let port = xport ^ ((STUN_MAGIC_COOKIE >> 16) as u16);
    let mut xip = [0u8; 4];
    xip.copy_from_slice(&value[4..8]);
    let cookie = STUN_MAGIC_COOKIE.to_be_bytes();
    for i in 0..4 {
        xip[i] ^= cookie[i];
    }
    // txid unused for IPv4 xor (RFC 5389); kept for API symmetry / IPv6 later
    let _ = txid;
    Ok(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(xip[0], xip[1], xip[2], xip[3])),
        port,
    ))
}

fn encode_xor_mapped(addr: SocketAddr, txid: &[u8; 12]) -> Result<Vec<u8>, PeerError> {
    let SocketAddr::V4(v4) = addr else {
        return Err(PeerError::Stun("mock mapped addr must be IPv4".into()));
    };
    let _ = txid;
    let mut out = vec![0u8; 8];
    out[1] = IPV4_FAMILY;
    let xport = v4.port() ^ ((STUN_MAGIC_COOKIE >> 16) as u16);
    out[2..4].copy_from_slice(&xport.to_be_bytes());
    let cookie = STUN_MAGIC_COOKIE.to_be_bytes();
    let octets = v4.ip().octets();
    for i in 0..4 {
        out[4 + i] = octets[i] ^ cookie[i];
    }
    Ok(out)
}

fn build_binding_success(txid: &[u8; 12], mapped: SocketAddr) -> Result<Vec<u8>, PeerError> {
    let attr_val = encode_xor_mapped(mapped, txid)?;
    let attr_len = attr_val.len() as u16;
    let mut msg = Vec::with_capacity(20 + 4 + attr_val.len());
    msg.extend_from_slice(&MSG_BINDING_SUCCESS.to_be_bytes());
    msg.extend_from_slice(&(4 + attr_val.len() as u16).to_be_bytes());
    msg.extend_from_slice(&STUN_MAGIC_COOKIE.to_be_bytes());
    msg.extend_from_slice(txid);
    msg.extend_from_slice(&ATTR_XOR_MAPPED_ADDRESS.to_be_bytes());
    msg.extend_from_slice(&attr_len.to_be_bytes());
    msg.extend_from_slice(&attr_val);
    Ok(msg)
}

/// Query a STUN server for the reflexive address (UDP Binding).
pub fn query_stun_reflexive(stun_server: &str, timeout: Duration) -> Result<SocketAddr, PeerError> {
    let server: SocketAddr = stun_server
        .parse()
        .map_err(|e| PeerError::Stun(format!("bad stun server {stun_server}: {e}")))?;
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| PeerError::Stun(e.to_string()))?;
    sock.set_read_timeout(Some(timeout / STUN_MAX_ATTEMPTS.max(1)))
        .map_err(|e| PeerError::Stun(e.to_string()))?;
    sock.connect(server)
        .map_err(|e| PeerError::Stun(e.to_string()))?;

    let mut txid = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut txid);
    let req = build_binding_request(&txid);

    let mut last_err = PeerError::Stun("STUN query failed".into());
    for _ in 0..STUN_MAX_ATTEMPTS {
        if let Err(e) = sock.send(&req) {
            last_err = PeerError::Stun(e.to_string());
            continue;
        }
        let mut buf = [0u8; 512];
        match sock.recv(&mut buf) {
            Ok(n) => match parse_binding_success(&buf[..n], &txid) {
                Ok(addr) => return Ok(addr),
                Err(e) => last_err = e,
            },
            Err(e) => last_err = PeerError::Stun(e.to_string()),
        }
    }
    Err(last_err)
}

/// Query STUN, persist [`StunReflexiveRecord`], return the record.
pub fn query_and_save_stun_reflexive(
    root: impl AsRef<Path>,
    stun_server: &str,
    timeout: Duration,
) -> Result<StunReflexiveRecord, PeerError> {
    let addr = query_stun_reflexive(stun_server, timeout)?;
    let observed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let rec = StunReflexiveRecord {
        addr: addr.to_string(),
        stun_server: stun_server.to_string(),
        observed_at,
    };
    rec.save(root)?;
    Ok(rec)
}

/// Resolve announce addr: exactly one of explicit `--addr` or `--from-stun`.
pub fn resolve_dht_announce_addr(
    root: impl AsRef<Path>,
    addr: Option<&str>,
    from_stun: bool,
) -> Result<String, PeerError> {
    match (addr, from_stun) {
        (Some(_), true) => Err(PeerError::Stun(
            "pass either --addr or --from-stun, not both (fail-closed)".into(),
        )),
        (None, false) => Err(PeerError::Stun(
            "dht announce requires --addr or --from-stun".into(),
        )),
        (Some(a), false) => {
            a.parse::<SocketAddr>()
                .map_err(|e| PeerError::Stun(format!("invalid addr {a}: {e}")))?;
            Ok(a.to_string())
        }
        (None, true) => {
            let rec = StunReflexiveRecord::load(root)?;
            rec.addr.parse::<SocketAddr>().map_err(|e| {
                PeerError::Stun(format!("bad stun_reflexive addr {}: {e}", rec.addr))
            })?;
            Ok(rec.addr)
        }
    }
}

/// In-process UDP mock STUN server for tests (IPv4).
///
/// Responds to Binding requests with XOR-MAPPED-ADDRESS set to `mapped`
/// (or the request source address when `mapped` is `None`).
pub struct MockStunServer {
    sock: UdpSocket,
    mapped: Option<SocketAddr>,
}

impl MockStunServer {
    pub fn bind(mapped: Option<SocketAddr>) -> Result<Self, PeerError> {
        let sock = UdpSocket::bind("127.0.0.1:0").map_err(|e| PeerError::Stun(e.to_string()))?;
        sock.set_read_timeout(Some(Duration::from_millis(200)))
            .map_err(|e| PeerError::Stun(e.to_string()))?;
        Ok(Self { sock, mapped })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, PeerError> {
        self.sock
            .local_addr()
            .map_err(|e| PeerError::Stun(e.to_string()))
    }

    /// Serve one Binding exchange (or return on timeout).
    pub fn serve_one(&self) -> Result<(), PeerError> {
        let mut buf = [0u8; 512];
        let (n, src) = self
            .sock
            .recv_from(&mut buf)
            .map_err(|e| PeerError::Stun(e.to_string()))?;
        if n < 20 {
            return Err(PeerError::Stun("mock: short request".into()));
        }
        let msg_type = u16::from_be_bytes([buf[0], buf[1]]);
        if msg_type != MSG_BINDING_REQUEST {
            return Err(PeerError::Stun("mock: not Binding request".into()));
        }
        let mut txid = [0u8; 12];
        txid.copy_from_slice(&buf[8..20]);
        let mapped = self.mapped.unwrap_or(src);
        let resp = build_binding_success(&txid, mapped)?;
        self.sock
            .send_to(&resp, src)
            .map_err(|e| PeerError::Stun(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tempfile::tempdir;

    #[test]
    fn rfc5769_xor_mapped_ipv4_vector() {
        // RFC 5769 §2.2 — XOR-MAPPED-ADDRESS value for 192.0.2.1:32853
        // with magic cookie 0x2112A442 (txid unused for IPv4).
        let value = hex::decode("0001a147e112a643").unwrap();
        let txid = [
            0xb7, 0xe7, 0xa7, 0x01, 0xbc, 0x34, 0xd6, 0x86, 0xfa, 0x87, 0xdf, 0xae,
        ];
        let addr = decode_xor_mapped(&value, &txid).unwrap();
        assert_eq!(addr, "192.0.2.1:32853".parse().unwrap());
    }

    #[test]
    fn mock_stun_roundtrip_and_persist() {
        let mapped: SocketAddr = "203.0.113.9:4040".parse().unwrap();
        let mock = MockStunServer::bind(Some(mapped)).unwrap();
        let server = mock.local_addr().unwrap();
        let server_s = server.to_string();
        let handle = thread::spawn(move || {
            mock.serve_one().unwrap();
        });
        // Give the server a moment to block on recv.
        thread::sleep(Duration::from_millis(20));
        let got = query_stun_reflexive(&server_s, Duration::from_secs(2)).unwrap();
        assert_eq!(got, mapped);
        handle.join().unwrap();

        let dir = tempdir().unwrap();
        let root = dir.path();
        let mock2 = MockStunServer::bind(Some(mapped)).unwrap();
        let server2 = mock2.local_addr().unwrap().to_string();
        let h2 = thread::spawn(move || mock2.serve_one().unwrap());
        thread::sleep(Duration::from_millis(20));
        let rec = query_and_save_stun_reflexive(root, &server2, Duration::from_secs(2)).unwrap();
        h2.join().unwrap();
        assert_eq!(rec.addr, mapped.to_string());
        let loaded = StunReflexiveRecord::load(root).unwrap();
        assert_eq!(loaded.addr, mapped.to_string());
    }

    #[test]
    fn resolve_from_stun_and_conflict() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        StunReflexiveRecord {
            addr: "198.51.100.1:9".into(),
            stun_server: "127.0.0.1:3478".into(),
            observed_at: 1,
        }
        .save(root)
        .unwrap();
        let a = resolve_dht_announce_addr(root, None, true).unwrap();
        assert_eq!(a, "198.51.100.1:9");
        let err = resolve_dht_announce_addr(root, Some("127.0.0.1:1"), true).unwrap_err();
        assert!(err.to_string().contains("not both"), "{err}");
        let err2 = resolve_dht_announce_addr(root, None, false).unwrap_err();
        assert!(err2.to_string().contains("requires"), "{err2}");
    }
}
