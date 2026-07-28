//! AIRA local node binary — load config, CSU registry, process local events / HTTP.

mod http;
mod tls;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::Parser;

use aira_csu::CsuRegistry;
use aira_flow::{init_node, load_config, LocalSession, SubmitOutcome, DEFAULT_AIRA_ROOT};
use aira_protocol::DiscoveryRegistry;

use crate::http::{router, AppState};
use crate::tls::{resolve_tls_paths, serve_https};

#[derive(Parser, Debug)]
#[command(
    name = "aira-node",
    version,
    about = "AIRA local node — load config/CSU and process local events"
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

    /// Shared secret for HTTP Bearer auth (Analyze-48). Also `AIRA_HTTP_TOKEN`.
    #[arg(long, env = "AIRA_HTTP_TOKEN")]
    http_token: Option<String>,
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
    if args.init || !args.root.join("config.json").exists() {
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
        return serve_http(
            args.root,
            &args.listen,
            args.tls_cert,
            args.tls_key,
            args.tls_self_signed,
            args.http_token,
        );
    }

    if args.tls_cert.is_some() || args.tls_key.is_some() || args.tls_self_signed {
        bail!("TLS flags require --http");
    }
    if args.http_token.is_some() {
        bail!("--http-token / AIRA_HTTP_TOKEN requires --http");
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

fn serve_http(
    root: PathBuf,
    listen: &str,
    tls_cert: Option<PathBuf>,
    tls_key: Option<PathBuf>,
    tls_self_signed: bool,
    http_token: Option<String>,
) -> Result<ExitCode> {
    let addr: SocketAddr = listen
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid --listen {listen}: {e}"))?;
    if !addr.ip().is_loopback() {
        eprintln!("warning: listening on non-loopback {addr} — M11 assumes local-only trust");
    }
    if let Some(ref t) = http_token {
        if t.trim().is_empty() {
            bail!("--http-token / AIRA_HTTP_TOKEN must be non-empty when set");
        }
    }
    let tls = resolve_tls_paths(&root, tls_cert, tls_key, tls_self_signed)?;
    let auth_enabled = http_token
        .as_ref()
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);
    let state = AppState::open(&root)
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .with_http_token(http_token);
    let app = router(state);
    println!(
        "discovery {}",
        DiscoveryRegistry::path(&root).display()
    );
    println!("endpoints: /health /v1/problems /v1/results /v1/artifacts /v1/events /v1/capabilities /v1/csu /v1/conformance/run");
    if auth_enabled {
        println!("http_auth: bearer enabled (/health exempt)");
    } else {
        println!("http_auth: off (loopback trust)");
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        if let Some((cert, key)) = tls {
            println!("https listening on https://{addr}");
            println!("tls_cert {}", cert.display());
            println!("tls_key {}", key.display());
            serve_https(addr, app, &cert, &key).await?;
        } else {
            println!("http listening on http://{addr}");
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
        Ok::<_, anyhow::Error>(())
    })?;
    Ok(ExitCode::SUCCESS)
}
