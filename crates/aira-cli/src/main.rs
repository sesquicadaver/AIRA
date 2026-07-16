//! AIRA CLI — local node, identity, CSU registry, problem/result/event commands.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

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
}

#[derive(Subcommand, Debug)]
enum IdentityCommands {
    /// Create Ed25519 keypair and local identity descriptor.
    Create {
        /// Identity display name / id suffix.
        #[arg(long, default_value = "local")]
        name: String,
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
            println!("status: C1 CLI / Local Node ready (Epic 8)");
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
                // Restrictive perms when possible (best-effort on Unix).
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        paths.identity_key(),
                        std::fs::Permissions::from_mode(0o600),
                    );
                }
                let identity_id = format!("aira:identity:{name}");
                let desc = serde_json::json!({
                    "identity_id": identity_id,
                    "identity_type": "local",
                    "display_name": name,
                    "public_key": {
                        "algorithm": "ed25519",
                        "key_hex": public_hex
                    },
                    "created_at": "2026-07-16T00:00:00Z",
                    "key_path": "identity/local.ed25519"
                });
                std::fs::write(paths.identity_json(), serde_json::to_string_pretty(&desc)?)?;
                println!("created {identity_id}");
                println!("public_key {public_hex}");
                println!("identity {}", paths.identity_json().display());
                Ok(ExitCode::SUCCESS)
            }
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
