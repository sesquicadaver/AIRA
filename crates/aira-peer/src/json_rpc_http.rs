//! Minimal JSON-RPC 2.0 HTTP client for EVM rendezvous (`#248`).
//!
//! Supports `http://` only (no TLS dependency / license surface). Live Amoy
//! HTTPS may use an HTTP gateway or local reference RPC in CI.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::{json, Value};

use crate::error::PeerError;

/// POST one JSON-RPC 2.0 call; returns the `result` field or maps `error`.
pub fn json_rpc_call(rpc_url: &str, method: &str, params: Value) -> Result<Value, PeerError> {
    let url = rpc_url.trim();
    if url.is_empty() {
        return Err(PeerError::Rendezvous("evm rpc_url empty".into()));
    }
    if url.starts_with("https://") {
        return Err(PeerError::Rendezvous(
            "live EVM JSON-RPC over https is not enabled in #248 (use http:// anvil/reference RPC or HTTP gateway)"
                .into(),
        ));
    }
    if !url.starts_with("http://") {
        return Err(PeerError::Rendezvous(format!(
            "live EVM RPC URL must be http://, got {url}"
        )));
    }
    let (host, port, path) = parse_http_url(url)?;
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    })
    .to_string();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let mut stream = TcpStream::connect((host.as_str(), port))
        .map_err(|e| PeerError::Rendezvous(format!("evm JSON-RPC connect: {e}")))?;
    stream.set_read_timeout(Some(Duration::from_secs(15))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(15))).ok();
    stream
        .write_all(req.as_bytes())
        .map_err(|e| PeerError::Rendezvous(format!("evm JSON-RPC write: {e}")))?;
    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .map_err(|e| PeerError::Rendezvous(format!("evm JSON-RPC read: {e}")))?;
    let text = std::str::from_utf8(&buf)
        .map_err(|e| PeerError::Rendezvous(format!("evm JSON-RPC utf8: {e}")))?;
    let body = http_response_body(text)?;
    let v: Value = serde_json::from_str(body)
        .map_err(|e| PeerError::Rendezvous(format!("evm JSON-RPC decode: {e}")))?;
    if let Some(err) = v.get("error") {
        return Err(PeerError::Rendezvous(format!("evm JSON-RPC error: {err}")));
    }
    v.get("result")
        .cloned()
        .ok_or_else(|| PeerError::Rendezvous("evm JSON-RPC missing result".into()))
}

fn parse_http_url(url: &str) -> Result<(String, u16, String), PeerError> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| PeerError::Rendezvous("expected http://".into()))?;
    let (authority, path) = match rest.split_once('/') {
        Some((a, p)) => (a, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    if authority.is_empty() {
        return Err(PeerError::Rendezvous("evm RPC host empty".into()));
    }
    let (host, port) = if let Some((h, p)) = authority.rsplit_once(':') {
        let port: u16 = p
            .parse()
            .map_err(|_| PeerError::Rendezvous(format!("bad RPC port: {p}")))?;
        (h.to_string(), port)
    } else {
        (authority.to_string(), 80)
    };
    Ok((host, port, path))
}

fn http_response_body(resp: &str) -> Result<&str, PeerError> {
    let idx = resp
        .find("\r\n\r\n")
        .ok_or_else(|| PeerError::Rendezvous("evm JSON-RPC malformed HTTP".into()))?;
    let (headers, body) = resp.split_at(idx + 4);
    let status_ok = headers
        .lines()
        .next()
        .map(|l| l.contains(" 200 ") || l.ends_with(" 200"))
        .unwrap_or(false);
    if !status_ok {
        return Err(PeerError::Rendezvous(format!(
            "evm JSON-RPC HTTP status: {}",
            headers.lines().next().unwrap_or("?")
        )));
    }
    Ok(body)
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
                            .and_then(|v| v.trim().parse::<usize>().ok())
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

    #[test]
    fn rejects_https_in_248() {
        let err = json_rpc_call(
            "https://rpc-amoy.polygon.technology/",
            "eth_chainId",
            json!([]),
        )
        .unwrap_err();
        assert!(err.to_string().contains("https"));
    }
}
