//! Loopback HTTP readiness probe (`GET /health`).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use anyhow::{bail, Context, Result};

/// Return true when `GET /health` responds with HTTP 200.
pub fn health_ok(listen: &str, timeout: Duration) -> Result<bool> {
    let addr = resolve_listen(listen)?;
    let mut stream =
        TcpStream::connect_timeout(&addr, timeout).with_context(|| format!("connect {listen}"))?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let host = listen.split(':').next().unwrap_or("127.0.0.1");
    let req = format!("GET /health HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes())?;
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf);
    Ok(text.starts_with("HTTP/1.1 200") || text.starts_with("HTTP/1.0 200"))
}

/// True if TCP connect succeeds (port occupied by something).
pub fn port_in_use(listen: &str, timeout: Duration) -> bool {
    resolve_listen(listen)
        .ok()
        .and_then(|addr| TcpStream::connect_timeout(&addr, timeout).ok())
        .is_some()
}

fn resolve_listen(listen: &str) -> Result<SocketAddr> {
    let mut iter = listen
        .to_socket_addrs()
        .with_context(|| format!("parse listen {listen}"))?;
    iter.next()
        .ok_or_else(|| anyhow::anyhow!("no address for {listen}"))
}

pub fn wait_healthy(listen: &str, overall: Duration) -> Result<()> {
    let start = std::time::Instant::now();
    let step = Duration::from_millis(100);
    while start.elapsed() < overall {
        if health_ok(listen, step).unwrap_or(false) {
            return Ok(());
        }
        std::thread::sleep(step);
    }
    bail!("timed out waiting for /health on {listen}");
}
