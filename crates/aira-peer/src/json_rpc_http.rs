//! Minimal JSON-RPC 2.0 HTTP client for EVM rendezvous (`#248`).
//!
//! Supports `http://` and `https://` via `ureq`. Used when
//! [`crate::evm_rendezvous::EvmRendezvousConfig::use_local_double`] is false.

use serde_json::{json, Value};

use crate::error::PeerError;

/// POST one JSON-RPC 2.0 call; returns the `result` field or maps `error`.
pub fn json_rpc_call(rpc_url: &str, method: &str, params: Value) -> Result<Value, PeerError> {
    let url = rpc_url.trim();
    if url.is_empty() {
        return Err(PeerError::Rendezvous("evm rpc_url empty".into()));
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(PeerError::Rendezvous(format!(
            "live EVM RPC URL must be http(s)://, got {url}"
        )));
    }
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    let resp = ureq::post(url)
        .set("content-type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| PeerError::Rendezvous(format!("evm JSON-RPC transport: {e}")))?;
    let text = resp
        .into_string()
        .map_err(|e| PeerError::Rendezvous(format!("evm JSON-RPC read: {e}")))?;
    let v: Value = serde_json::from_str(&text)
        .map_err(|e| PeerError::Rendezvous(format!("evm JSON-RPC decode: {e}")))?;
    if let Some(err) = v.get("error") {
        return Err(PeerError::Rendezvous(format!("evm JSON-RPC error: {err}")));
    }
    v.get("result")
        .cloned()
        .ok_or_else(|| PeerError::Rendezvous("evm JSON-RPC missing result".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;

    fn spawn_echo_rpc() -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                let n = stream.read(&mut chunk).unwrap();
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
                if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header = std::str::from_utf8(&buf[..pos]).unwrap_or("");
                    let cl = header.lines().find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().ok())
                            .flatten()
                    });
                    if let Some(cl) = cl {
                        if buf.len() >= pos + 4 + cl {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            let req = String::from_utf8_lossy(&buf).into_owned();
            log2.lock().unwrap().push(req);
            let body = r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
        });
        (format!("http://{addr}"), log)
    }

    #[test]
    fn posts_json_rpc_and_reads_result() {
        let (url, log) = spawn_echo_rpc();
        let got = json_rpc_call(&url, "eth_chainId", json!([])).unwrap();
        assert_eq!(got, json!({"ok": true}));
        let req = log.lock().unwrap()[0].clone();
        assert!(req.contains("eth_chainId"));
        assert!(req.contains("POST"));
    }
}
