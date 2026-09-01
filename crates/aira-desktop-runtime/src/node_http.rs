//! Loopback HTTP to the supervised `aira-node` (problem submit).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::bootstrap::read_http_token;
use crate::health::resolve_listen;
use crate::paths::DesktopPaths;
use crate::settings::{resolve_token_path, DesktopSettings, HttpAuthMode};

const SUBMIT_TIMEOUT: Duration = Duration::from_secs(60);

/// POST `{ "text" }` to `/v1/problems` on the Desktop node.
pub fn submit_desktop_problem(
    paths: &DesktopPaths,
    settings: &DesktopSettings,
    text: &str,
) -> Result<Value> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        bail!("problem text must be non-empty");
    }
    let token = match settings.http_auth_mode {
        HttpAuthMode::BearerToken => {
            let token_path = resolve_token_path(paths, settings)?;
            Some(read_http_token(&token_path)?)
        }
        HttpAuthMode::DesktopIpc => None,
    };
    submit_problem_http(
        &settings.http_listen,
        token.as_deref(),
        trimmed,
        SUBMIT_TIMEOUT,
    )
}

/// POST `/v1/problems` to an already-listening node.
pub fn submit_problem_http(
    listen: &str,
    bearer_token: Option<&str>,
    text: &str,
    timeout: Duration,
) -> Result<Value> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        bail!("problem text must be non-empty");
    }
    let body = serde_json::to_vec(&serde_json::json!({ "text": trimmed }))?;
    let addr = resolve_listen(listen)?;
    let mut stream =
        TcpStream::connect_timeout(&addr, timeout).with_context(|| format!("connect {listen}"))?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let host = listen.rsplit_once(':').map(|(h, _)| h).unwrap_or(listen);
    let auth = match bearer_token {
        Some(t) if !t.is_empty() => format!("Authorization: Bearer {t}\r\n"),
        _ => String::new(),
    };
    let req = format!(
        "POST /v1/problems HTTP/1.1\r\nHost: {host}\r\n{auth}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let mut msg = req.into_bytes();
    msg.extend_from_slice(&body);
    stream.write_all(&msg)?;
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    let (status, payload) = parse_http_response(&buf)?;
    if !(200..300).contains(&status) {
        if let Ok(v) = serde_json::from_str::<Value>(&payload) {
            if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
                bail!("HTTP {status}: {err}");
            }
        }
        bail!("HTTP {status}: {payload}");
    }
    serde_json::from_str(&payload).with_context(|| format!("parse problem response: {payload}"))
}

fn parse_http_response(raw: &[u8]) -> Result<(u16, String)> {
    let text = std::str::from_utf8(raw).context("HTTP response not UTF-8")?;
    let (head, body) = text
        .split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
        .ok_or_else(|| anyhow::anyhow!("malformed HTTP response"))?;
    let status_line = head.lines().next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("no HTTP status: {status_line}"))?;
    Ok((status, body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn empty_text_fails_closed() {
        let err = submit_problem_http("127.0.0.1:1", None, "   ", Duration::from_millis(50))
            .unwrap_err()
            .to_string();
        assert!(err.contains("non-empty"), "{err}");
    }

    #[test]
    fn posts_json_and_parses_completed() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            loop {
                let mut chunk = [0u8; 1024];
                let n = s.read(&mut chunk).unwrap();
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
                if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                    let req = String::from_utf8_lossy(&buf);
                    if let Some((_, rest)) = req.split_once("\r\n\r\n") {
                        if let Some(cl) = req
                            .lines()
                            .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
                            .and_then(|l| l.split(':').nth(1))
                            .and_then(|v| v.trim().parse::<usize>().ok())
                        {
                            if rest.len() >= cl {
                                break;
                            }
                        }
                    }
                }
            }
            let req = String::from_utf8_lossy(&buf);
            assert!(req.contains("POST /v1/problems"));
            assert!(req.contains("Authorization: Bearer tok"));
            assert!(req.contains(r#""text":"Calculate 2 + 2""#));
            let body = r#"{"status":"completed","result":4.0}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            s.write_all(resp.as_bytes()).unwrap();
        });
        let listen = format!("{}:{}", addr.ip(), addr.port());
        let v = submit_problem_http(
            &listen,
            Some("tok"),
            "Calculate 2 + 2",
            Duration::from_secs(2),
        )
        .unwrap();
        assert_eq!(v["status"], "completed");
        handle.join().unwrap();
    }

    #[test]
    fn http_error_surface_json_error() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 2048];
            let _ = s.read(&mut buf);
            let body = r#"{"error":"text must be non-empty"}"#;
            let resp = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            s.write_all(resp.as_bytes()).unwrap();
        });
        let listen = format!("{}:{}", addr.ip(), addr.port());
        let err = submit_problem_http(&listen, None, "x", Duration::from_secs(2))
            .unwrap_err()
            .to_string();
        assert!(err.contains("400"), "{err}");
        assert!(err.contains("text must be non-empty"), "{err}");
        handle.join().unwrap();
    }
}
