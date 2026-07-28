//! AIRA CLI — local node, identity, CSU registry, problem/result/event commands.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use aira_conformance::{run_profile, ConformanceProfile};
use aira_csu::{CsuLifecycleState, CsuManifest, CsuRegistry};
use aira_flow::{init_node, LocalSession, NodePaths, SubmitOutcome, DEFAULT_AIRA_ROOT};
use aira_schema::{find_repo_root, SchemaRegistry};

#[derive(Parser, Debug)]
#[command(
    name = "aira",
    version,
    about = "AIRA CLI — Problem Statement → Verified Result Artifact"
)]
struct Cli {
    /// Local node root (default: .aira).
    #[arg(long, global = true, default_value = DEFAULT_AIRA_ROOT)]
    root: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize local node layout (.aira).
    Init,
    /// Print bootstrap / runtime status.
    Status,
    /// Local identity commands.
    Identity {
        #[command(subcommand)]
        command: IdentityCommands,
    },
    /// Schema registry commands.
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
    /// Local CSU registry commands.
    Csu {
        #[command(subcommand)]
        command: CsuCommands,
    },
    /// Problem submit / status.
    Problem {
        #[command(subcommand)]
        command: ProblemCommands,
    },
    /// Fetch verified result payload.
    Result {
        #[command(subcommand)]
        command: ResultCommands,
    },
    /// Fetch artifact descriptor + payload.
    Artifact {
        #[command(subcommand)]
        command: ArtifactCommands,
    },
    /// Tail local event log.
    Event {
        #[command(subcommand)]
        command: EventCommands,
    },
    /// Authenticated peer links (Analyze-32/33 — no controlling center).
    Peer {
        #[command(subcommand)]
        command: PeerCommands,
    },
    /// Conformance suite runners (C0/C1).
    Conformance {
        #[command(subcommand)]
        command: ConformanceCommands,
    },
}

#[derive(Subcommand, Debug)]
enum IdentityCommands {
    /// Create Ed25519 keypair and local identity descriptor.
    Create {
        /// Identity display name / id suffix.
        #[arg(long, default_value = "local")]
        name: String,
    },
    /// Rotate node signing secret (same identity_id, new key material).
    Rotate {
        /// Persist previous secret to `identity/local.ed25519.prev` before overwrite.
        #[arg(long, default_value_t = false)]
        backup: bool,
        /// Keep previous pubkey verifiable for the same key_ref until this RFC3339 UTC instant.
        #[arg(long)]
        until: Option<String>,
        /// After rotate, best-effort notify address-book peers of the new pubkey (Analyze-38).
        #[arg(long, default_value_t = false)]
        notify_peers: bool,
    },
    /// Sign a message with the node identity key.
    Sign {
        /// Message bytes as UTF-8 text.
        #[arg(long)]
        text: String,
    },
    /// Verify a hex signature over a message with the node (or local-test) keyring.
    Verify {
        /// Message bytes as UTF-8 text.
        #[arg(long)]
        text: String,
        /// Hex-encoded Ed25519 signature.
        #[arg(long)]
        signature: String,
        /// key_ref (default: identity from node file, else local-test).
        #[arg(long)]
        key_ref: Option<String>,
    },
    /// Manage trusted verifying public keys.
    Trust {
        #[command(subcommand)]
        command: TrustCommands,
    },
}

#[derive(Subcommand, Debug)]
enum TrustCommands {
    /// List trusted identity public keys (and revoked CRL).
    List,
    /// Add or update a trusted verifying key.
    Add {
        #[arg(long)]
        key_ref: String,
        #[arg(long)]
        pubkey_hex: String,
    },
    /// Remove a trusted identity (not durable — can re-add).
    Remove {
        #[arg(long)]
        key_ref: String,
    },
    /// Durably revoke an identity (CRL — blocks re-add).
    Revoke {
        #[arg(long)]
        key_ref: String,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Clear CRL entry (does not auto re-trust — run trust add after).
    Unrevoke {
        #[arg(long)]
        key_ref: String,
    },
    /// Atomically revoke old peer and trust a new key_ref.
    ///
    /// Optional `--until` (RFC3339 UTC) keeps old key verifiable until that instant.
    Rotate {
        #[arg(long)]
        old_key_ref: String,
        #[arg(long)]
        new_key_ref: String,
        #[arg(long)]
        pubkey_hex: String,
        #[arg(long)]
        reason: Option<String>,
        /// Dual-key grace end (RFC3339 UTC, e.g. 2026-07-17T00:00:00Z).
        #[arg(long)]
        until: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum SchemaCommands {
    /// List registered schema `$id` values.
    List {
        #[arg(long)]
        schemas_dir: Option<PathBuf>,
    },
    /// Validate a JSON file or the fixture suite.
    Validate {
        #[arg(long)]
        schema: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        fixtures: Option<PathBuf>,
        #[arg(long)]
        schemas_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum CsuCommands {
    /// List registered CSU from the local registry file.
    List {
        #[arg(long)]
        registry: Option<PathBuf>,
    },
    /// Register a CSU manifest into the local registry.
    Register {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        registry: Option<PathBuf>,
        #[arg(long)]
        activate: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ProblemCommands {
    /// Submit a problem statement and run the local pipeline.
    Submit {
        #[arg(long)]
        text: String,
    },
    /// Show status for a problem ref.
    Status { problem_ref: String },
}

#[derive(Subcommand, Debug)]
enum ResultCommands {
    /// Get verified result JSON by problem-ref or artifact-ref.
    Get { result_ref: String },
}

#[derive(Subcommand, Debug)]
enum ArtifactCommands {
    /// Get artifact descriptor (and optionally raw payload).
    Get {
        artifact_ref: String,
        /// Print raw payload bytes as UTF-8 / hex instead of descriptor+JSON body.
        #[arg(long)]
        raw: bool,
    },
}

#[derive(Subcommand, Debug)]
enum EventCommands {
    /// Print the last N events from the local event log.
    Tail {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
enum PeerCommands {
    /// Upsert a peer into the static address book (must already be trusted).
    Add {
        #[arg(long)]
        key_ref: String,
        /// Socket address, e.g. 127.0.0.1:7900
        #[arg(long)]
        addr: String,
    },
    /// List address-book peers.
    List,
    /// Listen (loopback). Default: persistent accept loop (daemon).
    ///
    /// Without `--recv`, only hello is completed (dial smoke works). Use `--recv`
    /// to receive one envelope per accepted peer. Use `--once` for a single accept.
    Listen {
        #[arg(long, default_value = "127.0.0.1:0")]
        bind: String,
        /// Exit after one successful accept (and optional recv).
        #[arg(long, default_value_t = false)]
        once: bool,
        /// After hello, receive one envelope from the peer.
        #[arg(long, default_value_t = false)]
        recv: bool,
        /// When receiving `peer.trust.delta`, apply into local trust.json (fail-closed).
        #[arg(long, default_value_t = false)]
        apply_trust: bool,
    },
    /// Dial a trusted peer and complete hello.
    Dial {
        #[arg(long)]
        key_ref: String,
    },
    /// Dial a peer and send one signed peer.ping envelope.
    Send {
        #[arg(long)]
        key_ref: String,
        #[arg(long)]
        text: String,
    },
    /// Dial a peer and send one signed peer.trust.delta (Analyze-36).
    TrustSend {
        #[arg(long)]
        key_ref: String,
        /// revoke | rotate | unrevoke | rekey
        #[arg(long)]
        op: String,
        /// Subject (revoke/unrevoke/rekey) or old identity (rotate).
        #[arg(long)]
        subject: String,
        #[arg(long)]
        reason: Option<String>,
        /// Successor identity (rotate).
        #[arg(long)]
        new_id: Option<String>,
        /// Successor / new pubkey hex (rotate / rekey).
        #[arg(long)]
        pubkey_hex: Option<String>,
        /// Optional grace until RFC3339 UTC (rotate / rekey informational).
        #[arg(long)]
        until: Option<String>,
    },
    /// Notify one peer (or all address-book peers) of an upcoming pubkey rekey (Analyze-38).
    ///
    /// Call **before** `identity rotate` (or pass `--pubkey-hex` of the key you are about to
    /// install). Hello must still verify under the peer's current trust entry.
    NotifyRekey {
        /// If set, notify only this peer; otherwise all address-book peers.
        #[arg(long)]
        key_ref: Option<String>,
        /// New Ed25519 public key hex (64 chars) peers should trust.
        #[arg(long)]
        pubkey_hex: String,
        /// Optional grace_until forwarded in the rekey payload (informational).
        #[arg(long)]
        until: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ConformanceCommands {
    /// Run a conformance profile suite and emit a report artifact.
    Run {
        /// Profile: C0 or C1.
        #[arg(long, default_value = "C0")]
        profile: String,
        /// Directory for suite artifacts / report (default: <root>/conformance/reports).
        #[arg(long)]
        out: Option<PathBuf>,
    },
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
    let cli = Cli::parse();
    let root = cli.root;
    match cli.command {
        Commands::Init => {
            let paths = init_node(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("initialized {}", paths.root.display());
            println!("config {}", paths.config().display());
            println!("sqlite {}", paths.sqlite().display());
            println!("artifacts {}", paths.artifacts().display());
            Ok(ExitCode::SUCCESS)
        }
        Commands::Status => {
            println!("aira {}", env!("CARGO_PKG_VERSION"));
            println!("status: C1 Conformance ready (Epic 9)");
            if root.join("config.json").exists() {
                let session = LocalSession::open(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                println!(
                    "node: mode={} profile={}",
                    session.config.node.mode, session.config.node.profile
                );
                println!("root: {}", session.paths.root.display());
                let has_id = session.paths.identity_json().exists();
                println!("identity: {}", if has_id { "present" } else { "missing" });
            } else {
                println!(
                    "root: {} (not initialized — run `aira init`)",
                    root.display()
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Identity { command } => match command {
            IdentityCommands::Create { name } => {
                ensure_init(&root)?;
                let paths = NodePaths::new(&root);
                let mut rng = OsRng;
                let signing = SigningKey::generate(&mut rng);
                let verifying: VerifyingKey = signing.verifying_key();
                let secret_hex = hex::encode(signing.to_bytes());
                let public_hex = hex::encode(verifying.to_bytes());
                std::fs::create_dir_all(paths.identity_dir())?;
                std::fs::write(paths.identity_key(), format!("{secret_hex}\n"))?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        paths.identity_key(),
                        std::fs::Permissions::from_mode(0o600),
                    );
                }
                let identity_id = format!("aira:identity:{name}");
                let id_ref = aira_object::AiraRef::parse(&identity_id)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                let sig =
                    aira_object::sign_with_key(id_ref.clone(), &signing, identity_id.as_bytes());
                let desc = serde_json::json!({
                    "identity_id": identity_id,
                    "identity_type": "local",
                    "display_name": name,
                    "public_key": {
                        "algorithm": "ed25519",
                        "key_hex": public_hex
                    },
                    "created_at": "2026-07-16T00:00:00Z",
                    "key_path": "identity/local.ed25519",
                    "signature": sig
                });
                std::fs::write(paths.identity_json(), serde_json::to_string_pretty(&desc)?)?;
                let mut ring = aira_object::Keyring::with_local_test();
                ring.insert_signing(id_ref.clone(), signing);
                aira_object::register_keyring(&ring);
                aira_object::set_primary_signer(id_ref);
                let _ = aira_object::ensure_trust_defaults(&root);
                println!("created {identity_id}");
                println!("public_key {public_hex}");
                println!("identity {}", paths.identity_json().display());
                Ok(ExitCode::SUCCESS)
            }
            IdentityCommands::Rotate {
                backup,
                until,
                notify_peers,
            } => {
                ensure_init(&root)?;
                let mut rng = OsRng;
                let signing = SigningKey::generate(&mut rng);
                let new_pub = hex::encode(signing.verifying_key().to_bytes());
                // Notify *before* cutover so peers can still verify hello with the old pubkey.
                if notify_peers {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .context("tokio runtime")?;
                    let results = rt
                        .block_on(aira_peer::notify_peers_of_rekey(
                            &root,
                            &new_pub,
                            until.as_deref(),
                        ))
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    if results.is_empty() {
                        println!("notify_peers (empty address book)");
                    } else {
                        for r in results {
                            if r.ok {
                                println!("notified {}", r.peer_id);
                            } else {
                                eprintln!(
                                    "notify failed {}\t{}",
                                    r.peer_id,
                                    r.error.unwrap_or_default()
                                );
                            }
                        }
                    }
                }
                let (id, reported_new, old_pub, backup_path) =
                    aira_object::rotate_node_signing_secret(
                        &root,
                        signing,
                        backup,
                        until.as_deref(),
                    )
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("rotated {}", id.as_str());
                println!("old_public_key {old_pub}");
                println!("public_key {reported_new}");
                if let Some(until) = until.as_deref() {
                    println!("grace_until {until}");
                }
                if let Some(path) = backup_path {
                    println!("backup {}", path.display());
                }
                println!(
                    "identity {}",
                    NodePaths::new(&root).identity_json().display()
                );
                Ok(ExitCode::SUCCESS)
            }
            IdentityCommands::Sign { text } => {
                ensure_init(&root)?;
                let id = aira_object::register_node_identity(&root)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .ok_or_else(|| {
                        anyhow::anyhow!("no identity — run `aira identity create` first")
                    })?;
                let ring = aira_object::process_keyring_snapshot();
                let sig = ring
                    .sign(&id, text.as_bytes())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("{}", serde_json::to_string_pretty(&sig)?);
                Ok(ExitCode::SUCCESS)
            }
            IdentityCommands::Verify {
                text,
                signature,
                key_ref,
            } => {
                ensure_init(&root)?;
                let node_id = aira_object::register_node_identity(&root)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                let key_ref = match key_ref {
                    Some(k) => {
                        aira_object::AiraRef::parse(k).map_err(|e| anyhow::anyhow!("{e}"))?
                    }
                    None => node_id.unwrap_or_else(|| {
                        aira_object::AiraRef::parse(aira_object::LOCAL_TEST_KEY_REF).unwrap()
                    }),
                };
                let sig = aira_object::Signature {
                    algorithm: "ed25519".into(),
                    key_ref,
                    signature_value: signature,
                };
                match aira_object::verify_ed25519(&sig, text.as_bytes()) {
                    Ok(()) => {
                        println!("OK: signature valid");
                        Ok(ExitCode::SUCCESS)
                    }
                    Err(e) => {
                        eprintln!("FAIL: {e}");
                        Ok(ExitCode::FAILURE)
                    }
                }
            }
            IdentityCommands::Trust { command } => match command {
                TrustCommands::List => {
                    ensure_init(&root)?;
                    let store = aira_object::ensure_trust_defaults(&root)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    if store.entries.is_empty() {
                        println!("(empty trusted)");
                    } else {
                        for e in &store.entries {
                            println!("{}\t{}\t{}", e.identity_id, e.algorithm, e.public_key_hex);
                        }
                    }
                    if !store.revoked.is_empty() {
                        println!("# revoked");
                        for r in &store.revoked {
                            let reason = r.reason.as_deref().unwrap_or("-");
                            println!("REVOKED\t{}\t{}", r.identity_id, reason);
                        }
                    }
                    println!("trust {}", NodePaths::new(&root).trust_json().display());
                    Ok(ExitCode::SUCCESS)
                }
                TrustCommands::Add {
                    key_ref,
                    pubkey_hex,
                } => {
                    ensure_init(&root)?;
                    let mut store =
                        aira_object::TrustStore::load(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    store
                        .upsert(&key_ref, &pubkey_hex)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    store.save(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    aira_object::register_trust_store(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    println!("trusted {key_ref}");
                    Ok(ExitCode::SUCCESS)
                }
                TrustCommands::Remove { key_ref } => {
                    ensure_init(&root)?;
                    let mut store =
                        aira_object::TrustStore::load(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    if key_ref == aira_object::LOCAL_TEST_KEY_REF {
                        bail!("refusing to remove local-test from trust store");
                    }
                    if store.remove(&key_ref) {
                        store.save(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                        aira_object::sync_trust_verifiers(&root)
                            .map_err(|e| anyhow::anyhow!("{e}"))?;
                        println!("removed {key_ref}");
                    } else {
                        println!("not found {key_ref}");
                    }
                    Ok(ExitCode::SUCCESS)
                }
                TrustCommands::Revoke { key_ref, reason } => {
                    ensure_init(&root)?;
                    let mut store =
                        aira_object::TrustStore::load(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    store
                        .revoke(&key_ref, reason.as_deref())
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    store.save(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    aira_object::sync_trust_verifiers(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    println!("revoked {key_ref}");
                    Ok(ExitCode::SUCCESS)
                }
                TrustCommands::Unrevoke { key_ref } => {
                    ensure_init(&root)?;
                    let mut store =
                        aira_object::TrustStore::load(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    store
                        .unrevoke(&key_ref)
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    store.save(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    println!("unrevoked {key_ref} (not trusted until `trust add`)");
                    Ok(ExitCode::SUCCESS)
                }
                TrustCommands::Rotate {
                    old_key_ref,
                    new_key_ref,
                    pubkey_hex,
                    reason,
                    until,
                } => {
                    ensure_init(&root)?;
                    let mut store =
                        aira_object::TrustStore::load(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    store
                        .rotate(
                            &old_key_ref,
                            &new_key_ref,
                            &pubkey_hex,
                            reason.as_deref(),
                            until.as_deref(),
                        )
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    store.save(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    aira_object::sync_trust_verifiers(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                    match until {
                        Some(u) => {
                            println!("rotated {old_key_ref} -> {new_key_ref} (grace until {u})")
                        }
                        None => println!("rotated {old_key_ref} -> {new_key_ref}"),
                    }
                    Ok(ExitCode::SUCCESS)
                }
            },
        },
        Commands::Schema { command } => match command {
            SchemaCommands::List { schemas_dir } => {
                let reg = load_schema_registry(schemas_dir)?;
                for id in reg.list_ids() {
                    println!("{id}");
                }
                Ok(ExitCode::SUCCESS)
            }
            SchemaCommands::Validate {
                schema,
                file,
                fixtures,
                schemas_dir,
            } => {
                let reg = load_schema_registry(schemas_dir)?;
                if let Some(fixtures_root) = fixtures {
                    let root_repo = if fixtures_root.as_os_str() == "fixtures"
                        || fixtures_root.ends_with("fixtures")
                    {
                        find_repo_root(std::env::current_dir()?)?
                    } else {
                        fixtures_root
                    };
                    let report = reg.validate_fixtures(&root_repo)?;
                    println!(
                        "fixtures: passed={} failed={}",
                        report.passed, report.failed
                    );
                    for f in &report.failures {
                        eprintln!("FAIL: {f}");
                    }
                    if report.failed > 0 {
                        return Ok(ExitCode::FAILURE);
                    }
                    return Ok(ExitCode::SUCCESS);
                }

                let schema = schema.context("--schema is required unless --fixtures is set")?;
                let file = file.context("--file is required unless --fixtures is set")?;
                match reg.validate_file(&schema, &file) {
                    Ok(()) => {
                        println!("OK: {} validates against {schema}", file.display());
                        Ok(ExitCode::SUCCESS)
                    }
                    Err(e) => {
                        eprintln!("FAIL: {e}");
                        Ok(ExitCode::FAILURE)
                    }
                }
            }
        },
        Commands::Csu { command } => match command {
            CsuCommands::List { registry } => {
                let path = registry.unwrap_or_else(|| default_csu_registry(&root));
                if !path.exists() {
                    println!("(empty) no registry at {}", path.display());
                    return Ok(ExitCode::SUCCESS);
                }
                let reg = CsuRegistry::load(&path).map_err(|e| anyhow::anyhow!("{e}"))?;
                for entry in reg.list() {
                    println!(
                        "{}\t{:?}\t{:?}\t{}",
                        entry.manifest.csu_id,
                        entry.state,
                        entry.manifest.csu_type,
                        entry.manifest.csu_name
                    );
                }
                Ok(ExitCode::SUCCESS)
            }
            CsuCommands::Register {
                manifest,
                registry,
                activate,
            } => {
                let path = registry.unwrap_or_else(|| default_csu_registry(&root));
                let mut reg = if path.exists() {
                    CsuRegistry::load(&path).map_err(|e| anyhow::anyhow!("{e}"))?
                } else {
                    CsuRegistry::new()
                };
                let text = std::fs::read_to_string(&manifest)
                    .with_context(|| format!("read {}", manifest.display()))?;
                let m: CsuManifest = serde_json::from_str(&text)
                    .with_context(|| format!("parse {}", manifest.display()))?;
                if let Ok(schema_reg) = load_schema_registry(None) {
                    let v = serde_json::to_value(&m)?;
                    schema_reg
                        .validate("aira:schema:csu:manifest:0.1", &v)
                        .map_err(|e| anyhow::anyhow!("manifest schema: {e}"))?;
                }
                let id = m.csu_id.clone();
                reg.register(m, None)
                    .map_err(|e| anyhow::anyhow!("register: {e}"))?;
                if activate {
                    reg.activate(&id, None)
                        .map_err(|e| anyhow::anyhow!("activate: {e}"))?;
                }
                reg.save(&path)
                    .map_err(|e| anyhow::anyhow!("save registry: {e}"))?;
                let state = reg
                    .get(&id)
                    .map(|e| e.state)
                    .unwrap_or(CsuLifecycleState::Registered);
                println!("registered {} state={state:?}", id);
                println!("registry {}", path.display());
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Problem { command } => match command {
            ProblemCommands::Submit { text } => {
                ensure_init(&root)?;
                let mut session = LocalSession::open(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                let out = session
                    .submit_problem(&text)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                match out {
                    SubmitOutcome::Completed {
                        problem_id,
                        verified_artifact_id,
                        result,
                    } => {
                        println!("problem_ref {}", problem_id);
                        println!("result_ref {}", verified_artifact_id);
                        println!("status completed");
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    }
                    SubmitOutcome::NeedsHumanCollapse { field_artifact_id } => {
                        let pref = session
                            .plane()
                            .problem_ref()
                            .map(|r| r.as_str().to_string())
                            .unwrap_or_default();
                        println!("problem_ref {pref}");
                        println!("field_ref {}", field_artifact_id);
                        println!("status needs_human_collapse");
                    }
                }
                Ok(ExitCode::SUCCESS)
            }
            ProblemCommands::Status { problem_ref } => {
                ensure_init(&root)?;
                let session = LocalSession::open(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                let rec = session
                    .problem_status(&problem_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("{}", serde_json::to_string_pretty(&rec)?);
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Result { command } => match command {
            ResultCommands::Get { result_ref } => {
                ensure_init(&root)?;
                let session = LocalSession::open(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                let v = session
                    .get_result(&result_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("{}", serde_json::to_string_pretty(&v)?);
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Artifact { command } => match command {
            ArtifactCommands::Get { artifact_ref, raw } => {
                ensure_init(&root)?;
                let session = LocalSession::open(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                let (desc, bytes) = session
                    .get_artifact(&artifact_ref)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                if raw {
                    match std::str::from_utf8(&bytes) {
                        Ok(s) => print!("{s}"),
                        Err(_) => println!("{}", hex::encode(&bytes)),
                    }
                } else {
                    let mut out = serde_json::Map::new();
                    out.insert("descriptor".into(), desc);
                    if let Ok(body) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        out.insert("payload".into(), body);
                    } else {
                        out.insert("payload_hex".into(), serde_json::json!(hex::encode(&bytes)));
                    }
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::Value::Object(out))?
                    );
                }
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Event { command } => match command {
            EventCommands::Tail { limit } => {
                ensure_init(&root)?;
                let session = LocalSession::open(&root).map_err(|e| anyhow::anyhow!("{e}"))?;
                let events = session
                    .event_tail(limit)
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                for e in events {
                    println!(
                        "{}\t{:?}\t{}",
                        e.event_id,
                        e.event_type,
                        e.payload_ref.as_deref().unwrap_or("-")
                    );
                }
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Peer { command } => {
            ensure_init(&root)?;
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("tokio runtime")?;
            rt.block_on(run_peer(&root, command))
        }
        Commands::Conformance { command } => match command {
            ConformanceCommands::Run { profile, out } => {
                let profile = match profile.to_uppercase().as_str() {
                    "C0" => ConformanceProfile::C0,
                    "C1" => ConformanceProfile::C1,
                    other => bail!("unsupported profile {other} (use C0 or C1)"),
                };
                let out = out.unwrap_or_else(|| root.join("conformance").join("reports"));
                std::fs::create_dir_all(&out)?;
                let suite = run_profile(profile, &out).map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("profile {}", suite.report.aira.profile.as_str());
                println!(
                    "results total={} passed={} failed={} skipped={}",
                    suite.report.results.total,
                    suite.report.results.passed,
                    suite.report.results.failed,
                    suite.report.results.skipped
                );
                println!("report_artifact {}", suite.report_artifact_id);
                for f in &suite.report.failures {
                    eprintln!("FAIL {}: {}", f.test_id, f.reason);
                }
                if suite.report.results.failed > 0 {
                    Ok(ExitCode::FAILURE)
                } else {
                    Ok(ExitCode::SUCCESS)
                }
            }
        },
    }
}

fn ensure_init(root: &Path) -> Result<()> {
    if !root.join("config.json").exists() {
        bail!(
            "node not initialized at {} — run `aira init --root {}`",
            root.display(),
            root.display()
        );
    }
    Ok(())
}

fn require_trusted(root: &Path, key_ref: &str) -> Result<()> {
    let store = aira_object::TrustStore::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
    if store.is_revoked(key_ref) {
        bail!("peer identity revoked: {key_ref}");
    }
    if !store.entries.iter().any(|e| e.identity_id == key_ref) {
        bail!("peer not trusted — run `aira identity trust add` first: {key_ref}");
    }
    Ok(())
}

fn build_peer_ping(root: &Path, text: &str) -> Result<aira_protocol::ProtocolEnvelope> {
    aira_peer::make_peer_ping(root, text).map_err(|e| anyhow::anyhow!("{e}"))
}

async fn run_peer(root: &Path, command: PeerCommands) -> Result<ExitCode> {
    match command {
        PeerCommands::Add { key_ref, addr } => {
            require_trusted(root, &key_ref)?;
            addr.parse::<std::net::SocketAddr>()
                .with_context(|| format!("invalid addr {addr}"))?;
            let mut book =
                aira_peer::AddressBook::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            book.upsert(&key_ref, &addr);
            book.save(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("peer {key_ref} -> {addr}");
            println!(
                "address_book {}",
                aira_peer::AddressBook::path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        PeerCommands::List => {
            let book = aira_peer::AddressBook::load(root).map_err(|e| anyhow::anyhow!("{e}"))?;
            if book.peers.is_empty() {
                println!("(empty address book)");
            } else {
                for p in &book.peers {
                    println!("{}\t{}", p.identity_id, p.addr);
                }
            }
            println!(
                "address_book {}",
                aira_peer::AddressBook::path(root).display()
            );
            Ok(ExitCode::SUCCESS)
        }
        PeerCommands::Listen {
            bind,
            once,
            recv,
            apply_trust,
        } => {
            if apply_trust && !recv {
                bail!("--apply-trust requires --recv");
            }
            let listener = aira_peer::listen(&bind)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let addr = listener.local_addr()?;
            println!("listening {addr}");
            if once {
                println!("mode once");
            } else {
                println!("mode daemon");
            }
            if recv {
                println!("recv enabled");
            }
            if apply_trust {
                println!("apply_trust enabled");
            }
            let root_owned = root.to_path_buf();
            loop {
                let mut peer = match aira_peer::accept(&listener, root).await {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("accept error: {e}");
                        if once {
                            return Err(anyhow::anyhow!("{e}"));
                        }
                        // Avoid tight spin if the listener is wedged; admission errors
                        // still return to waiting on the next TCP accept.
                        if matches!(e, aira_peer::PeerError::Io(_)) {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                        continue;
                    }
                };
                println!("accepted {}", peer.peer_id.as_str());
                if recv {
                    if once {
                        let env = peer
                            .recv_envelope()
                            .await
                            .map_err(|e| anyhow::anyhow!("{e}"))?;
                        println!(
                            "received {}\t{}\t{}",
                            env.message_type,
                            env.message_id.as_str(),
                            env.issuer_identity.as_str()
                        );
                        if let Some(payload) = env.payload_ref.as_deref() {
                            println!("payload_ref {payload}");
                        }
                        if apply_trust && env.message_type == aira_peer::TRUST_DELTA_MESSAGE_TYPE {
                            let delta = aira_peer::parse_trust_delta(&env)
                                .map_err(|e| anyhow::anyhow!("{e}"))?;
                            aira_peer::apply_trust_delta(root, &env.issuer_identity, &delta)
                                .map_err(|e| anyhow::anyhow!("{e}"))?;
                            println!(
                                "applied trust-delta {:?}\tsubject {}",
                                delta.op, delta.subject_id
                            );
                        }
                        break;
                    }
                    // Daemon: recv off the accept path so the next hello is not blocked.
                    let root_bg = root_owned.clone();
                    let do_apply = apply_trust;
                    tokio::spawn(async move {
                        match peer.recv_envelope().await {
                            Ok(env) => {
                                println!(
                                    "received {}\t{}\t{}",
                                    env.message_type,
                                    env.message_id.as_str(),
                                    env.issuer_identity.as_str()
                                );
                                if let Some(payload) = env.payload_ref.as_deref() {
                                    println!("payload_ref {payload}");
                                }
                                if do_apply
                                    && env.message_type == aira_peer::TRUST_DELTA_MESSAGE_TYPE
                                {
                                    match aira_peer::parse_trust_delta(&env).and_then(|d| {
                                        aira_peer::apply_trust_delta(
                                            &root_bg,
                                            &env.issuer_identity,
                                            &d,
                                        )
                                        .map(|_| d)
                                    }) {
                                        Ok(delta) => println!(
                                            "applied trust-delta {:?}\tsubject {}",
                                            delta.op, delta.subject_id
                                        ),
                                        Err(e) => eprintln!(
                                            "apply_trust error from {}: {e}",
                                            env.issuer_identity.as_str()
                                        ),
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("recv error from {}: {e}", peer.peer_id.as_str());
                            }
                        }
                    });
                    continue;
                }
                if once {
                    break;
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        PeerCommands::Dial { key_ref } => {
            require_trusted(root, &key_ref)?;
            let peer = aira_peer::dial(root, &key_ref)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("dialed {}", peer.peer_id.as_str());
            println!("local {}", peer.local_id.as_str());
            Ok(ExitCode::SUCCESS)
        }
        PeerCommands::Send { key_ref, text } => {
            require_trusted(root, &key_ref)?;
            let env = build_peer_ping(root, &text)?;
            let mut peer = aira_peer::dial(root, &key_ref)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            peer.send_envelope(&env)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!(
                "sent {}\t{}\t-> {}",
                env.message_type,
                env.message_id.as_str(),
                peer.peer_id.as_str()
            );
            Ok(ExitCode::SUCCESS)
        }
        PeerCommands::TrustSend {
            key_ref,
            op,
            subject,
            reason,
            new_id,
            pubkey_hex,
            until,
        } => {
            require_trusted(root, &key_ref)?;
            let op = aira_peer::TrustDeltaOp::parse(&op).map_err(|e| anyhow::anyhow!("{e}"))?;
            let delta = match op {
                aira_peer::TrustDeltaOp::Revoke => aira_peer::TrustDelta::revoke(subject, reason),
                aira_peer::TrustDeltaOp::Unrevoke => aira_peer::TrustDelta::unrevoke(subject),
                aira_peer::TrustDeltaOp::Rotate => {
                    let new_id =
                        new_id.ok_or_else(|| anyhow::anyhow!("rotate requires --new-id"))?;
                    let pubkey_hex = pubkey_hex
                        .ok_or_else(|| anyhow::anyhow!("rotate requires --pubkey-hex"))?;
                    aira_peer::TrustDelta::rotate(subject, new_id, pubkey_hex, reason, until)
                }
                aira_peer::TrustDeltaOp::Rekey => {
                    let pubkey_hex =
                        pubkey_hex.ok_or_else(|| anyhow::anyhow!("rekey requires --pubkey-hex"))?;
                    aira_peer::TrustDelta::rekey(subject, pubkey_hex, reason, until)
                }
            };
            let env = aira_peer::make_trust_delta_envelope(root, &delta)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let mut peer = aira_peer::dial(root, &key_ref)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            peer.send_envelope(&env)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!(
                "sent {}\t{:?}\t{}\t-> {}",
                env.message_type,
                delta.op,
                delta.subject_id,
                peer.peer_id.as_str()
            );
            Ok(ExitCode::SUCCESS)
        }
        PeerCommands::NotifyRekey {
            key_ref,
            pubkey_hex,
            until,
        } => {
            if let Some(key_ref) = key_ref {
                require_trusted(root, &key_ref)?;
                aira_peer::notify_peer_of_rekey(
                    root,
                    &key_ref,
                    &pubkey_hex,
                    until.as_deref(),
                )
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("notified {key_ref}");
            } else {
                let results =
                    aira_peer::notify_peers_of_rekey(root, &pubkey_hex, until.as_deref())
                        .await
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                if results.is_empty() {
                    println!("notify_rekey (empty address book)");
                } else {
                    for r in results {
                        if r.ok {
                            println!("notified {}", r.peer_id);
                        } else {
                            eprintln!(
                                "notify failed {}\t{}",
                                r.peer_id,
                                r.error.unwrap_or_default()
                            );
                        }
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn default_csu_registry(root: &Path) -> PathBuf {
    let modern = root.join("csu").join("registry.json");
    if modern.exists() {
        return modern;
    }
    // Legacy fallback from Epic 5 CLI.
    let legacy = PathBuf::from(".aira/csu-registry.json");
    if legacy.exists() {
        return legacy;
    }
    modern
}

fn load_schema_registry(schemas_dir: Option<PathBuf>) -> Result<SchemaRegistry> {
    let dir = if let Some(d) = schemas_dir {
        d
    } else {
        let root = find_repo_root(std::env::current_dir()?)
            .or_else(|_| find_repo_root(env!("CARGO_MANIFEST_DIR")).context("locate repo root"))?;
        root.join("schemas")
    };
    if !dir.is_dir() {
        bail!("schemas dir not found: {}", dir.display());
    }
    SchemaRegistry::load(dir)
}
