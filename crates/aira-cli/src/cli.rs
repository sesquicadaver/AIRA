//! Clap surface for `aira` (Analyze-81). Mechanical extract from `main.rs`.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use aira_flow::DEFAULT_AIRA_ROOT;

#[derive(Parser, Debug)]
#[command(
    name = "aira",
    version,
    about = "AIRA CLI — Problem Statement → Verified Result Artifact"
)]
pub(crate) struct Cli {
    /// Local node root (default: .aira).
    #[arg(long, global = true, default_value = DEFAULT_AIRA_ROOT)]
    pub(crate) root: PathBuf,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
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
    /// Local federation join (Analyze-70) — operator ceremony, not a network handshake.
    Federation {
        #[command(subcommand)]
        command: FederationCommands,
    },
    /// Conformance suite runners (C0/C1).
    Conformance {
        #[command(subcommand)]
        command: ConformanceCommands,
    },
    /// Local model inventory (Phase D — read-only scan/list; no download).
    Models {
        #[command(subcommand)]
        command: ModelsCommands,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum IdentityCommands {
    /// Create Ed25519 keypair and local identity descriptor.
    Create {
        /// Identity display name / id suffix.
        #[arg(long, default_value = "local")]
        name: String,
    },
    /// Rotate node signing secret (same identity_id, new key material).
    Rotate {
        /// Persist previous secret to `identity/local.ed25519.prev` before overwrite.
        /// Prior `.prev` is archived under a UTC timestamp (Analyze-41).
        #[arg(long, default_value_t = false)]
        backup: bool,
        /// Keep previous pubkey verifiable for the same key_ref until this RFC3339 UTC instant.
        #[arg(long)]
        until: Option<String>,
        /// After rotate, best-effort notify address-book peers of the new pubkey (Analyze-38).
        #[arg(long, default_value_t = false)]
        notify_peers: bool,
    },
    /// List durable node signing-secret backups (latest + timestamped history).
    /// Subcommand `prune` GCs archived `.prev.<stamp>` slots (Analyze-61).
    Backups {
        #[command(subcommand)]
        command: Option<BackupsCommands>,
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
    /// Durable per-CSU signing secrets (Analyze-62).
    CsuTenant {
        #[command(subcommand)]
        command: CsuTenantCommands,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum CsuTenantCommands {
    /// Generate (or import) a tenant signing key, save under identity/tenants/, register.
    Register {
        #[arg(long)]
        csu_id: String,
        #[arg(long)]
        publisher: String,
        /// Optional 64-hex Ed25519 seed; default: generate random.
        /// Demo-only (visible in process list / shell history); prefer `--secret-hex-file`.
        #[arg(long)]
        secret_hex: Option<String>,
        /// 64-hex Ed25519 seed from a file (`-` = stdin, not a TTY). Prefer over `--secret-hex`.
        /// A file named `-` must be passed as `./-`. Mutually exclusive with `--secret-hex`.
        #[arg(long, conflicts_with = "secret_hex")]
        secret_hex_file: Option<String>,
        /// Overwrite an existing durable tenant dir (prefer `rotate`).
        #[arg(long)]
        force: bool,
    },
    /// List durable tenant dirs on disk.
    List,
    /// Load all durable tenants into process memory.
    Load,
    /// Rotate tenant signing secret (same publisher_id).
    Rotate {
        #[arg(long)]
        csu_id: String,
        /// Keep previous secret as `ed25519.prev` (archives prior latest).
        #[arg(long)]
        backup: bool,
        /// Optional 64-hex Ed25519 seed; default: generate random (demo-only if set).
        /// Prefer `--secret-hex-file`.
        #[arg(long)]
        secret_hex: Option<String>,
        /// 64-hex Ed25519 seed from a file (`-` = stdin, not a TTY). Prefer over `--secret-hex`.
        /// A file named `-` must be passed as `./-`. Mutually exclusive with `--secret-hex`.
        #[arg(long, conflicts_with = "secret_hex")]
        secret_hex_file: Option<String>,
    },
    /// Revoke tenant: unload + delete dir + audit (signing-side; not TrustStore CRL).
    Revoke {
        #[arg(long)]
        csu_id: String,
        #[arg(long)]
        reason: String,
    },
    /// List durable tenant `ed25519.prev` slots; subcommand `prune` GCs archives (Analyze-71).
    Backups {
        #[command(subcommand)]
        command: Option<CsuTenantBackupsCommands>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum CsuTenantBackupsCommands {
    /// Prune archived tenant `ed25519.prev.<stamp>` slots (never latest `.prev`).
    Prune {
        /// Keep at most N newest archived slots per tenant.
        #[arg(long)]
        keep: Option<u64>,
        /// Drop archives older than D days (per tenant).
        #[arg(long)]
        older_than_days: Option<u64>,
        /// Report deletes without removing files.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum BackupsCommands {
    /// Prune archived `.prev.<stamp>` slots for ed25519 + x25519 (never latest `.prev`).
    Prune {
        /// Keep at most N newest archived slots per family.
        #[arg(long)]
        keep: Option<u64>,
        /// Drop archives older than D days (per family age rules).
        #[arg(long)]
        older_than_days: Option<u64>,
        /// Report deletes without removing files.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum TrustCommands {
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
    /// List durable trust/ceremony audit entries (`identity/trust-audit.jsonl`).
    Audit {
        /// Show only the last N entries (default: all).
        #[arg(long)]
        last: Option<usize>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum SchemaCommands {
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
pub(crate) enum CsuCommands {
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
pub(crate) enum ProblemCommands {
    /// Submit a problem statement and run the local pipeline.
    Submit {
        #[arg(long)]
        text: String,
    },
    /// Show status for a problem ref.
    Status { problem_ref: String },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ResultCommands {
    /// Get verified result JSON by problem-ref or artifact-ref.
    Get { result_ref: String },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ArtifactCommands {
    /// Get artifact descriptor (and optionally raw payload).
    Get {
        artifact_ref: String,
        /// Print raw payload bytes as UTF-8 / hex instead of descriptor+JSON body.
        #[arg(long)]
        raw: bool,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum EventCommands {
    /// Print the last N events from the local event log.
    Tail {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum PeerCommands {
    /// Upsert a peer into the static address book (must already be trusted).
    Add {
        #[arg(long)]
        key_ref: String,
        /// Socket address, e.g. 127.0.0.1:7900
        #[arg(long)]
        addr: String,
        /// Optional courier relay identity (Analyze-44). Send/trust-send use deliver via this peer.
        #[arg(long)]
        via: Option<String>,
    },
    /// List address-book peers.
    List,
    /// Listen (loopback). Default: persistent accept loop (daemon).
    ///
    /// Without `--recv`, only hello is completed (dial smoke works). Use `--recv`
    /// to receive one envelope per accepted peer. Use `--once` for a single accept.
    /// Daemon mode (Analyze-59): TCP accept stays on the loop; hello/Noise (+recv/relay)
    /// run on per-connection tasks so a slow handshake cannot block further accepts.
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
        /// After apply, fan out the original trust-delta to other address-book peers (Analyze-43).
        #[arg(long, default_value_t = false)]
        gossip: bool,
        /// Run as relay hub: register live sessions and forward `peer.relay.deliver` (Analyze-44).
        #[arg(long, default_value_t = false)]
        relay: bool,
        /// Optional TTL days for durable offline registry prune (Analyze-58). Requires `--relay`.
        /// Recommended: 31. Omit = keep offline history forever.
        #[arg(long)]
        relay_ttl_days: Option<u64>,
        /// Apply inbound `peer.dht.announce` into local DHT table (Analyze-47).
        #[arg(long, default_value_t = false)]
        dht: bool,
        /// After DHT announce apply, also upsert identity/addr into address book (Analyze-57).
        /// Requires `--dht`. Preserves existing `via`.
        #[arg(long, default_value_t = false)]
        apply_book: bool,
    },
    /// List durable peer discovery journal (`peers/discovery.json`).
    Discovery,
    /// Trusted-mesh DHT-lite (Analyze-47).
    Dht {
        #[command(subcommand)]
        command: PeerDhtCommands,
    },
    /// STUN Binding reflexive discovery (Analyze-66). Does not change `dial`.
    Stun {
        #[command(subcommand)]
        command: PeerStunCommands,
    },
    /// UDP discv5-style announce (Analyze-67). Store only; no FIND_NODE / apply-book.
    Discv {
        #[command(subcommand)]
        command: PeerDiscvCommands,
    },
    /// Hold an outbound session to a relay (register for inbound delivers).
    RelayHold {
        #[arg(long)]
        key_ref: String,
        /// Apply inbound `peer.trust.delta` (courier-delivered) into local trust.json.
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
pub(crate) enum PeerDhtCommands {
    /// Announce local listen addr into DHT and fan out to address-book peers.
    Announce {
        /// Dialable socket addr to advertise (e.g. 127.0.0.1:7900).
        #[arg(long)]
        addr: Option<String>,
        /// Use addr from `peers/stun_reflexive.json` (Analyze-66). Mutually exclusive with `--addr`.
        #[arg(long, default_value_t = false)]
        from_stun: bool,
    },
    /// Find closest DHT records for an identity (local table).
    Find {
        #[arg(long)]
        key_ref: String,
        #[arg(long, default_value_t = aira_peer::DHT_DEFAULT_K)]
        k: usize,
        /// Upsert exact DHT hit into address book (Analyze-57). Preserves existing `via`.
        #[arg(long, default_value_t = false)]
        apply_book: bool,
    },
    /// List local DHT records.
    List,
}

#[derive(Subcommand, Debug)]
pub(crate) enum PeerStunCommands {
    /// RFC 5389 Binding query → print + write `peers/stun_reflexive.json`.
    Query {
        /// STUN server `host:port` (required; also `AIRA_STUN_SERVER`). No public default.
        #[arg(long, env = "AIRA_STUN_SERVER")]
        stun_server: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum PeerDiscvCommands {
    /// Bind UDP and apply inbound signed announces into `peers/dht.json` (`source=udp`).
    Listen {
        #[arg(long, default_value = "127.0.0.1:0")]
        bind: String,
        /// Exit after one successful store.
        #[arg(long, default_value_t = false)]
        once: bool,
        /// Allow non-loopback bind (operator).
        #[arg(long, default_value_t = false)]
        explicit: bool,
    },
    /// Send one signed UDP announce to a discv listener.
    Announce {
        /// Destination UDP `host:port`.
        #[arg(long)]
        to: String,
        /// Dialable TCP addr to advertise (xor `--from-stun`).
        #[arg(long)]
        addr: Option<String>,
        /// Use `peers/stun_reflexive.json` (Analyze-66).
        #[arg(long, default_value_t = false)]
        from_stun: bool,
    },
    /// Iterative XOR FIND_NODE over UDP discv (Analyze-68). Merges trusted hints into dht.json.
    Find {
        #[arg(long)]
        key_ref: String,
        /// Optional extra UDP seed (`host:port`).
        #[arg(long)]
        to: Option<String>,
        #[arg(long, default_value_t = aira_peer::DHT_DEFAULT_K)]
        k: usize,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum FederationCommands {
    /// Pin a self-signed federation descriptor (local membership + TrustStore).
    Join {
        /// Path to signed descriptor JSON.
        #[arg(long)]
        descriptor: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConformanceCommands {
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

#[derive(Subcommand, Debug)]
pub(crate) enum ModelsCommands {
    /// Scan scoped `<root>/models` (or `--dir` under it) and publish inventory artifact.
    Scan {
        /// Directory to scan (must stay under `<root>/models`). Default: `<root>/models`.
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// List the latest local model inventory snapshot (no network).
    List,
    /// Classify installed models (runnable/incompatible/unknown); publish evidence (no download).
    Compatible,
    /// Show local acquisition policy (default-deny if absent).
    Policy {
        #[command(subcommand)]
        command: ModelsPolicyCommands,
    },
    /// Request model download — D3 always DENY (no real transfer; D4 is Out).
    Download {
        #[arg(long)]
        model_ref: String,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum ModelsPolicyCommands {
    /// Print effective acquisition policy.
    Show,
    /// Write acquisition policy (`--auto-download` sets true; default false).
    Set {
        #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
        auto_download: bool,
    },
}
