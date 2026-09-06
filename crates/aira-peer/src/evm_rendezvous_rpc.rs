//! Reference EVM rendezvous JSON-RPC HTTP surface (QUEUE #248 / Phase N-fix).
//!
//! Not a full Ethereum node. Implements AIRA rendezvous methods over HTTP
//! JSON-RPC so [`crate::evm_rendezvous::EvmRendezvousProvider`] with
//! `use_local_double=false` can dial a real socket in CI (anvil-shaped stand-in).
//! Presence authenticity remains AIRA Ed25519 — never an EVM payer.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use serde_json::{json, Value};

use crate::error::PeerError;
use crate::presence::NodePresenceRecord;
use crate::rendezvous::{MockRendezvousProvider, RendezvousProvider};

/// JSON-RPC method: publish signed presence.
pub const RPC_PUBLISH: &str = "aira_rendezvous_publish";
/// JSON-RPC method: update signed presence (monotonic sequence).
pub const RPC_UPDATE: &str = "aira_rendezvous_update";
/// JSON-RPC method: remove or expire by identity.
pub const RPC_REMOVE: &str = "aira_rendezvous_remove";
/// JSON-RPC method: list active peers at `as_of`.
pub const RPC_QUERY_ACTIVE: &str = "aira_rendezvous_queryActive";
/// JSON-RPC method: lookup by identity_ref.
pub const RPC_QUERY_IDENTITY: &str = "aira_rendezvous_queryIdentity";
/// JSON-RPC method: active peers with relay endpoints.
pub const RPC_QUERY_RELAYS: &str = "aira_rendezvous_queryRelays";
/// Standard eth chain id (hex string result).
pub const RPC_ETH_CHAIN_ID: &str = "eth_chainId";

/// In-process reference rendezvous RPC (Mock ledger + HTTP JSON-RPC).
#[derive(Debug)]
pub struct ReferenceEvmRendezvousRpc {
    addr: SocketAddr,
    _join: JoinHandle<()>,
    stop: Arc<Mutex<bool>>,
}

impl ReferenceEvmRendezvousRpc {
    /// Bind `127.0.0.1:0`, spawn accept loop, return handle + base URL.
    pub fn spawn(chain_id: u64, expected_contract: impl Into<String>) -> Result<Self, PeerError> {
        let expected_contract = expected_contract.into();
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|e| PeerError::Io(e.to_string()))?;
        listener
            .set_nonblocking(false)
            .map_err(|e| PeerError::Io(e.to_string()))?;
        let addr = listener
            .local_addr()
            .map_err(|e| PeerError::Io(e.to_string()))?;
        let stop = Arc::new(Mutex::new(false));
        let stop_thr = stop.clone();
        let ledger = Arc::new(Mutex::new(MockRendezvousProvider::new()));
        let join = thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            loop {
                if *stop_thr.lock().unwrap_or_else(|e| e.into_inner()) {
                    break;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        let ledger = ledger.clone();
                        let contract = expected_contract.clone();
                        let _ = handle_connection(stream, chain_id, &contract, &ledger);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });
        // Give the accept loop a moment; first client may arrive immediately.
        thread::sleep(std::time::Duration::from_millis(20));
        Ok(Self {
            addr,
            _join: join,
            stop,
        })
    }

    /// `http://127.0.0.1:port` for [`crate::evm_rendezvous::EvmRendezvousConfig::rpc_url`].
    pub fn rpc_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Bound address.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for ReferenceEvmRendezvousRpc {
    fn drop(&mut self) {
        if let Ok(mut g) = self.stop.lock() {
            *g = true;
        }
        // Touch the port so accept wakes; ignore errors.
        let _ = TcpStream::connect(self.addr);
    }
}

fn handle_connection(
    mut stream: TcpStream,
    chain_id: u64,
    expected_contract: &str,
    ledger: &Arc<Mutex<MockRendezvousProvider>>,
) -> Result<(), PeerError> {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = stream
            .read(&mut chunk)
            .map_err(|e| PeerError::Io(e.to_string()))?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            // If Content-Length present, wait for full body.
            if let Some(cl) = content_length(&buf) {
                if let Some(body_start) = find_body_start(&buf) {
                    if buf.len() - body_start >= cl {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        if buf.len() > 2 * 1024 * 1024 {
            return Err(PeerError::Rendezvous("rpc request too large".into()));
        }
    }
    let body = http_body(&buf).unwrap_or("");
    let response_body = dispatch_rpc(body, chain_id, expected_contract, ledger);
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    stream
        .write_all(resp.as_bytes())
        .map_err(|e| PeerError::Io(e.to_string()))?;
    Ok(())
}

fn content_length(req: &[u8]) -> Option<usize> {
    let s = std::str::from_utf8(req).ok()?;
    for line in s.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("content-length:") {
            return rest.trim().parse().ok();
        }
    }
    None
}

fn find_body_start(req: &[u8]) -> Option<usize> {
    req.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4)
}

fn http_body(req: &[u8]) -> Option<&str> {
    let start = find_body_start(req)?;
    std::str::from_utf8(&req[start..]).ok()
}

fn dispatch_rpc(
    body: &str,
    chain_id: u64,
    expected_contract: &str,
    ledger: &Arc<Mutex<MockRendezvousProvider>>,
) -> String {
    let req: Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => {
            return json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": { "code": -32700, "message": format!("parse error: {e}") }
            })
            .to_string();
        }
    };
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    let params = req.get("params").cloned().unwrap_or(Value::Null);
    let result = match method.as_str() {
        RPC_ETH_CHAIN_ID => Ok(json!(format!("0x{chain_id:x}"))),
        RPC_PUBLISH => rpc_publish(params, expected_contract, ledger),
        RPC_UPDATE => rpc_update(params, expected_contract, ledger),
        RPC_REMOVE => rpc_remove(params, expected_contract, ledger),
        RPC_QUERY_ACTIVE => rpc_query_active(params, expected_contract, ledger),
        RPC_QUERY_IDENTITY => rpc_query_identity(params, expected_contract, ledger),
        RPC_QUERY_RELAYS => rpc_query_relays(params, expected_contract, ledger),
        other => Err(format!("method not found: {other}")),
    };
    match result {
        Ok(v) => json!({ "jsonrpc": "2.0", "id": id, "result": v }).to_string(),
        Err(msg) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32000, "message": msg }
        })
        .to_string(),
    }
}

fn require_contract(params: &Value, expected: &str) -> Result<(), String> {
    let got = params
        .get("contract")
        .and_then(|c| c.as_str())
        .ok_or_else(|| "missing contract".to_string())?;
    if !got.eq_ignore_ascii_case(expected) {
        return Err(format!("contract mismatch: expected {expected}, got {got}"));
    }
    Ok(())
}

fn parse_record(params: &Value) -> Result<NodePresenceRecord, String> {
    let rec = params
        .get("record")
        .ok_or_else(|| "missing record".to_string())?;
    serde_json::from_value(rec.clone()).map_err(|e| e.to_string())
}

fn rpc_publish(
    params: Value,
    expected_contract: &str,
    ledger: &Arc<Mutex<MockRendezvousProvider>>,
) -> Result<Value, String> {
    require_contract(&params, expected_contract)?;
    let record = parse_record(&params)?;
    let mut g = ledger.lock().map_err(|_| "ledger lock".to_string())?;
    g.publish_presence(record).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

fn rpc_update(
    params: Value,
    expected_contract: &str,
    ledger: &Arc<Mutex<MockRendezvousProvider>>,
) -> Result<Value, String> {
    require_contract(&params, expected_contract)?;
    let record = parse_record(&params)?;
    let mut g = ledger.lock().map_err(|_| "ledger lock".to_string())?;
    g.update_presence(record).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

fn rpc_remove(
    params: Value,
    expected_contract: &str,
    ledger: &Arc<Mutex<MockRendezvousProvider>>,
) -> Result<Value, String> {
    require_contract(&params, expected_contract)?;
    let identity_ref = params
        .get("identity_ref")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing identity_ref".to_string())?;
    let as_of = params
        .get("as_of")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing as_of".to_string())?;
    let force = params
        .get("force")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let mut g = ledger.lock().map_err(|_| "ledger lock".to_string())?;
    let removed = g
        .remove_or_expire_presence(identity_ref, as_of, force)
        .map_err(|e| e.to_string())?;
    Ok(json!({ "removed": removed }))
}

fn rpc_query_active(
    params: Value,
    expected_contract: &str,
    ledger: &Arc<Mutex<MockRendezvousProvider>>,
) -> Result<Value, String> {
    require_contract(&params, expected_contract)?;
    let as_of = params
        .get("as_of")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing as_of".to_string())?;
    let g = ledger.lock().map_err(|_| "ledger lock".to_string())?;
    let peers = g.query_active_peers(as_of).map_err(|e| e.to_string())?;
    serde_json::to_value(peers).map_err(|e| e.to_string())
}

fn rpc_query_identity(
    params: Value,
    expected_contract: &str,
    ledger: &Arc<Mutex<MockRendezvousProvider>>,
) -> Result<Value, String> {
    require_contract(&params, expected_contract)?;
    let identity_ref = params
        .get("identity_ref")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing identity_ref".to_string())?;
    let g = ledger.lock().map_err(|_| "ledger lock".to_string())?;
    let rec = g.query_identity(identity_ref).map_err(|e| e.to_string())?;
    serde_json::to_value(rec).map_err(|e| e.to_string())
}

fn rpc_query_relays(
    params: Value,
    expected_contract: &str,
    ledger: &Arc<Mutex<MockRendezvousProvider>>,
) -> Result<Value, String> {
    require_contract(&params, expected_contract)?;
    let as_of = params
        .get("as_of")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing as_of".to_string())?;
    let g = ledger.lock().map_err(|_| "ledger lock".to_string())?;
    let peers = g.query_relays(as_of).map_err(|e| e.to_string())?;
    serde_json::to_value(peers).map_err(|e| e.to_string())
}
