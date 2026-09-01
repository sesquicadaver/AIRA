//! AIRA local node binary — load config, CSU registry, process local events / HTTP.

mod http;
mod tenant_auth;
mod tls;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::Parser;

use aira_csu::CsuRegistry;
use aira_flow::{
    init_node, load_config, node_config_present, LocalSession, SubmitOutcome, DEFAULT_AIRA_ROOT,
};
use aira_protocol::DiscoveryRegistry;

use crate::http::{health_router, router, AppState};
use crate::tenant_auth::{default_tenant_auth_path, validate_http_auth_boot};
use crate::tls::{resolve_tls_paths, serve_https};

#[derive(Parser, Debug)]
#[command(
    name = "aira-node",
    version,
    about = "AIRA local node — load config/CSU and process local events",
    after_help = "Examples:\n  aira-node --http --listen 127.0.0.1:8787\n  aira-node --http --allow-public-bind --listen 0.0.0.0:8787 --http-token \"$TOKEN\"\n\nNon-loopback bind is fail-closed without --allow-public-bind. TLS/Bearer stay opt-in."
)]
struct Args {
    /// Local node root (default: .aira).
    #[arg(long, default_value = DEFAULT_AIRA_ROOT)]
    root: PathBuf,

    /// Create layout if missing.
    #[arg(long)]
    init: bool,

    /// Process one problem statement then exit.
    #[arg(long)]
    text: Option<String>,

    /// Serve Roadmap M11 local HTTP API (blocks).
    #[arg(long)]
    http: bool,

    /// Listen address for `--http` (default loopback).
    #[arg(long, default_value = "127.0.0.1:8787")]
    listen: String,

    /// PEM certificate for HTTPS (requires `--tls-key`).
    #[arg(long)]
    tls_cert: Option<PathBuf>,

    /// PEM private key for HTTPS (requires `--tls-cert`).
    #[arg(long)]
    tls_key: Option<PathBuf>,

    /// Generate/reuse self-signed cert under `<root>/http/` (Analyze-45).
    #[arg(long, default_value_t = false)]
    tls_self_signed: bool,

    /// Client CA PEM — require client cert (mTLS, Analyze-51). Requires HTTPS.
    #[arg(long)]
    tls_client_ca: Option<PathBuf>,

    /// Shared secret for HTTP Bearer auth (Analyze-48). Also `AIRA_HTTP_TOKEN`.
    #[arg(long, env = "AIRA_HTTP_TOKEN")]
    http_token: Option<String>,

    /// Tenant Bearer→publisher map (Analyze-64). Default: `<root>/identity/http-tenant-auth.json` if present.
    #[arg(long)]
    http_tenant_auth: Option<PathBuf>,

    /// Plain-HTTP liveness bind (`GET /health` only). Requires mTLS on `--listen` (Analyze-56).
    #[arg(long)]
    health_listen: Option<String>,

    /// Allow non-loopback `--listen` / `--health-listen` (Analyze-69). Default: fail-closed.
    #[arg(long, default_value_t = false)]
    allow_public_bind: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let args = Args::parse();
    if args.init || !node_config_present(&args.root) {
        if args.init {
            let paths = init_node(&args.root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("initialized {}", paths.root.display());
        } else {
            bail!(
                "node not initialized at {} — pass --init or run `aira init`",
                args.root.display()
            );
        }
    }

    let config = load_config(&args.root).map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("aira-node {}", env!("CARGO_PKG_VERSION"));
    println!(
        "config: mode={} profile={}",
        config.node.mode, config.node.profile
    );
    println!("autoload: {}", config.csu.autoload.join(", "));

    let registry_path = args.root.join("csu").join("registry.json");
    if registry_path.exists() {
        let reg = CsuRegistry::load(&registry_path).map_err(|e| anyhow::anyhow!("{e}"))?;
        let entries = reg.list();
        println!(
            "csu_registry: {} entries at {}",
            entries.len(),
            registry_path.display()
        );
        for e in entries {
            println!("  {}\t{:?}", e.manifest.csu_id, e.state);
        }
    } else {
        println!("csu_registry: (empty) — built-in basic CSUs used by OperationalPlane");
    }

    if args.http {
        if args.text.is_some() {
            bail!("--http and --text are mutually exclusive");
        }
        return serve_http(HttpOpts {
            root: args.root,
            listen: args.listen,
            tls_cert: args.tls_cert,
            tls_key: args.tls_key,
            tls_self_signed: args.tls_self_signed,
            tls_client_ca: args.tls_client_ca,
            http_token: args.http_token,
            http_tenant_auth: args.http_tenant_auth,
            health_listen: args.health_listen,
            allow_public_bind: args.allow_public_bind,
        });
    }

    if args.tls_cert.is_some()
        || args.tls_key.is_some()
        || args.tls_self_signed
        || args.tls_client_ca.is_some()
        || args.health_listen.is_some()
        || args.allow_public_bind
    {
        bail!("TLS / mTLS / --health-listen / --allow-public-bind flags require --http");
    }
    if args.http_token.is_some() || args.http_tenant_auth.is_some() {
        bail!("--http-token / --http-tenant-auth require --http");
    }

    if let Some(text) = args.text {
        let mut session = LocalSession::open(&args.root).map_err(|e| anyhow::anyhow!("{e}"))?;
        let out = session
            .submit_problem(&text)
            .map_err(|e| anyhow::anyhow!("process: {e}"))?;
        match out {
            SubmitOutcome::Completed {
                problem_id,
                verified_artifact_id,
                result,
            } => {
                println!("processed problem_ref={problem_id}");
                println!("result_ref={verified_artifact_id}");
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            SubmitOutcome::Executed {
                problem_id,
                execution_artifact_id,
                result,
            } => {
                println!("processed problem_ref={problem_id}");
                println!("result_ref={execution_artifact_id}");
                println!("status executed");
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            SubmitOutcome::NeedsHumanCollapse { field_artifact_id } => {
                println!("needs_human_collapse field_ref={field_artifact_id}");
            }
        }
        let tail = session.event_tail(10).map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("event_tail ({})", tail.len());
        for e in tail {
            println!("  {}\t{:?}", e.event_id, e.event_type);
        }
    } else {
        println!("idle: pass --text \"Calculate 2 + 2\" or --http --listen 127.0.0.1:8787");
    }

    Ok(ExitCode::SUCCESS)
}

/// Fail-closed unless `addr` is loopback or `--allow-public-bind` was set.
fn assert_bind_allowed(addr: SocketAddr, allow_public: bool, flag: &str) -> Result<()> {
    if addr.ip().is_loopback() {
        return Ok(());
    }
    if !allow_public {
        bail!("{flag} {addr} is not loopback — pass --allow-public-bind (fail-closed default)");
    }
    Ok(())
}

/// Parse optional `--health-listen` (Analyze-56). Requires mTLS on the API listener.
fn resolve_health_listen(
    mtls: bool,
    health_listen: Option<&str>,
    allow_public: bool,
) -> Result<Option<SocketAddr>> {
    let Some(raw) = health_listen.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    if !mtls {
        bail!("--health-listen requires --tls-client-ca (mTLS on --listen)");
    }
    let addr: SocketAddr = raw
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid --health-listen {raw}: {e}"))?;
    assert_bind_allowed(addr, allow_public, "--health-listen")?;
    Ok(Some(addr))
}

/// Bundled `--http` options (avoids clippy `too_many_arguments`).
struct HttpOpts {
    root: PathBuf,
    listen: String,
    tls_cert: Option<PathBuf>,
    tls_key: Option<PathBuf>,
    tls_self_signed: bool,
    tls_client_ca: Option<PathBuf>,
    http_token: Option<String>,
    http_tenant_auth: Option<PathBuf>,
    health_listen: Option<String>,
    allow_public_bind: bool,
}

fn serve_http(opts: HttpOpts) -> Result<ExitCode> {
    let HttpOpts {
        root,
        listen,
        tls_cert,
        tls_key,
        tls_self_signed,
        tls_client_ca,
        http_token,
        http_tenant_auth,
        health_listen,
        allow_public_bind,
    } = opts;
    let addr: SocketAddr = listen
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid --listen {listen}: {e}"))?;
    assert_bind_allowed(addr, allow_public_bind, "--listen")?;
    if !addr.ip().is_loopback() && tls_cert.is_none() && tls_key.is_none() && !tls_self_signed {
        eprintln!(
            "warning: public bind {addr} without TLS — operator choice (TLS/Bearer remain opt-in)"
        );
    }
    if let Some(ref t) = http_token {
        if t.trim().is_empty() {
            bail!("--http-token / AIRA_HTTP_TOKEN must be non-empty when set");
        }
    }
    let explicit_map = http_tenant_auth.is_some();
    let map_path = http_tenant_auth.unwrap_or_else(|| default_tenant_auth_path(&root));
    let tenant_map = validate_http_auth_boot(http_token.as_deref(), &map_path, explicit_map)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let tenant_auth_on = tenant_map.is_some();
    let tls = resolve_tls_paths(&root, tls_cert, tls_key, tls_self_signed)?;
    if tls_client_ca.is_some() && tls.is_none() {
        bail!("--tls-client-ca requires HTTPS (--tls-cert/--tls-key or --tls-self-signed)");
    }
    if let Some(ref ca) = tls_client_ca {
        // Fail closed early (also validated again when building ServerConfig).
        let _ = crate::tls::load_client_ca_roots(ca)?;
    }
    let auth_enabled = http_token
        .as_ref()
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);
    let mtls = tls_client_ca.is_some();
    let health_addr = resolve_health_listen(mtls, health_listen.as_deref(), allow_public_bind)?;
    if let Some(haddr) = health_addr {
        if !haddr.ip().is_loopback() {
            eprintln!("warning: public --health-listen {haddr} is plaintext GET /health");
        }
    }
    let state = AppState::open(&root)
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .with_http_token(http_token)
        .with_tenant_auth(tenant_map);
    let app = router(state);
    println!("discovery {}", DiscoveryRegistry::path(&root).display());
    println!("endpoints: /health /v1/problems /v1/results /v1/artifacts /v1/events /v1/capabilities /v1/csu /v1/conformance/run");
    if auth_enabled {
        println!("http_auth: bearer enabled (/health exempt at HTTP layer)");
    } else if addr.ip().is_loopback() {
        println!("http_auth: off (loopback trust)");
    } else {
        println!("http_auth: off (public bind; TLS/Bearer remain opt-in)");
    }
    if tenant_auth_on {
        println!("http_tenant_auth {}", map_path.display());
    } else {
        println!("http_tenant_auth: off");
    }
    if mtls {
        if health_addr.is_some() {
            eprintln!(
                "warning: mTLS on API — client cert still required on --listen; plain probe is only on --health-listen"
            );
        } else {
            eprintln!(
                "warning: mTLS enabled — client certificate required for ALL routes including /health (unlike Bearer); set --health-listen for a plain probe"
            );
        }
        println!("mtls: require client cert; CN must match TrustStore AiraRef");
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let health_task = if let Some(haddr) = health_addr {
            println!("health listening on http://{haddr} (GET /health only, no client cert)");
            let hr = health_router();
            Some(tokio::spawn(async move {
                let listener = tokio::net::TcpListener::bind(haddr).await?;
                axum::serve(listener, hr).await?;
                Ok::<_, anyhow::Error>(())
            }))
        } else {
            None
        };

        let api = async {
            if let Some((cert, key)) = tls {
                println!("https listening on https://{addr}");
                println!("tls_cert {}", cert.display());
                println!("tls_key {}", key.display());
                if let Some(ref ca) = tls_client_ca {
                    println!("tls_client_ca {}", ca.display());
                }
                serve_https(addr, app, &cert, &key, tls_client_ca.as_deref(), &root).await
            } else {
                println!("http listening on http://{addr}");
                let listener = tokio::net::TcpListener::bind(addr).await?;
                axum::serve(listener, app).await?;
                Ok(())
            }
        };

        match health_task {
            Some(ht) => {
                tokio::select! {
                    r = api => r?,
                    r = ht => r.map_err(|e| anyhow::anyhow!("health task join: {e}"))??,
                }
            }
            None => api.await?,
        }
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod bind_policy_tests {
    use super::{assert_bind_allowed, resolve_health_listen};
    use std::net::SocketAddr;

    #[test]
    fn health_listen_none_ok() {
        assert!(resolve_health_listen(true, None, false).unwrap().is_none());
        assert!(resolve_health_listen(false, None, false).unwrap().is_none());
    }

    #[test]
    fn health_listen_requires_mtls() {
        let err = resolve_health_listen(false, Some("127.0.0.1:8788"), false).unwrap_err();
        assert!(
            err.to_string().contains("requires --tls-client-ca"),
            "{err}"
        );
    }

    #[test]
    fn health_listen_parses_loopback() {
        let addr = resolve_health_listen(true, Some("127.0.0.1:8788"), false)
            .unwrap()
            .unwrap();
        assert_eq!(addr.port(), 8788);
        assert!(addr.ip().is_loopback());
    }

    #[test]
    fn health_listen_rejects_non_loopback_without_flag() {
        let err = resolve_health_listen(true, Some("0.0.0.0:8788"), false).unwrap_err();
        assert!(err.to_string().contains("--allow-public-bind"), "{err}");
    }

    #[test]
    fn health_listen_allows_non_loopback_with_flag() {
        let addr = resolve_health_listen(true, Some("0.0.0.0:8788"), true)
            .unwrap()
            .unwrap();
        assert!(!addr.ip().is_loopback());
        assert_eq!(addr.port(), 8788);
    }

    #[test]
    fn listen_rejects_public_without_flag() {
        let addr: SocketAddr = "0.0.0.0:8787".parse().unwrap();
        let err = assert_bind_allowed(addr, false, "--listen").unwrap_err();
        assert!(err.to_string().contains("--allow-public-bind"), "{err}");
        assert!(err.to_string().contains("fail-closed"), "{err}");
    }

    #[test]
    fn listen_allows_loopback_without_flag() {
        let addr: SocketAddr = "127.0.0.1:8787".parse().unwrap();
        assert_bind_allowed(addr, false, "--listen").unwrap();
        let v6: SocketAddr = "[::1]:8787".parse().unwrap();
        assert_bind_allowed(v6, false, "--listen").unwrap();
    }

    #[test]
    fn listen_allows_public_with_flag() {
        let addr: SocketAddr = "0.0.0.0:8787".parse().unwrap();
        assert_bind_allowed(addr, true, "--listen").unwrap();
        let v6: SocketAddr = "[::]:8787".parse().unwrap();
        assert_bind_allowed(v6, true, "--listen").unwrap();
    }

    #[test]
    fn listen_rejects_unspecified_v6_without_flag() {
        let addr: SocketAddr = "[::]:8787".parse().unwrap();
        let err = assert_bind_allowed(addr, false, "--listen").unwrap_err();
        assert!(err.to_string().contains("--allow-public-bind"), "{err}");
    }

    #[test]
    fn health_listen_rejects_unspecified_v6_without_flag() {
        let err = resolve_health_listen(true, Some("[::]:8788"), false).unwrap_err();
        assert!(err.to_string().contains("--allow-public-bind"), "{err}");
    }
}
