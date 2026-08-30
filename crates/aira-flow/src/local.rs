//! Local node layout and session persistence (Epic 8 / #57–#62).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use aira_artifact::{ArtifactStore, CasArtifactStore};
use aira_core::SqliteObjectStore;
use aira_event::{EventDescriptor, FileChainEventLog};
use aira_object::{AiraRef, ContentHash};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::plane::{FlowError, OperationalPlane, SubmitOutcome};

/// Default relative root for a local node.
pub const DEFAULT_AIRA_ROOT: &str = ".aira";

/// Node configuration written by `aira init`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeConfig {
    pub node: NodeSection,
    pub security: SecuritySection,
    pub storage: StorageSection,
    pub csu: CsuSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeSection {
    pub mode: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecuritySection {
    pub allow_network_for_csu: bool,
    pub allow_shell_for_csu: bool,
    pub require_signed_artifacts: bool,
    pub require_signed_events: bool,
    pub require_signed_csu_manifests: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageSection {
    pub object_store: String,
    pub event_log: String,
    pub artifact_store: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CsuSection {
    pub autoload: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node: NodeSection {
                mode: "local".into(),
                profile: "C1".into(),
            },
            security: SecuritySection {
                allow_network_for_csu: false,
                allow_shell_for_csu: false,
                require_signed_artifacts: true,
                require_signed_events: true,
                require_signed_csu_manifests: true,
            },
            storage: StorageSection {
                object_store: "sqlite".into(),
                event_log: "json".into(),
                artifact_store: "filesystem".into(),
            },
            csu: CsuSection {
                autoload: vec![
                    "context-basic".into(),
                    "reduction-basic".into(),
                    "execution-basic".into(),
                    "verification-basic".into(),
                    "evidence-basic".into(),
                    "artifact-basic".into(),
                ],
            },
        }
    }
}

/// Paths under a local `.aira` root.
#[derive(Debug, Clone)]
pub struct NodePaths {
    pub root: PathBuf,
}

impl NodePaths {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Canonical JSON config path (`aira init` writer).
    pub fn config_json(&self) -> PathBuf {
        self.root.join("config.json")
    }

    /// Optional YAML config path (read parity with JSON).
    pub fn config_yaml(&self) -> PathBuf {
        self.root.join("config.yaml")
    }

    /// Alias for [`Self::config_json`] (writers / display).
    pub fn config(&self) -> PathBuf {
        self.config_json()
    }
    pub fn identity_dir(&self) -> PathBuf {
        self.root.join("identity")
    }
    pub fn identity_json(&self) -> PathBuf {
        self.identity_dir().join("local.identity.json")
    }
    pub fn identity_key(&self) -> PathBuf {
        self.identity_dir().join("local.ed25519")
    }
    pub fn trust_json(&self) -> PathBuf {
        self.identity_dir().join("trust.json")
    }
    pub fn trust_audit_jsonl(&self) -> PathBuf {
        self.identity_dir().join(aira_object::TRUST_AUDIT_FILE)
    }
    pub fn db_dir(&self) -> PathBuf {
        self.root.join("db")
    }
    pub fn sqlite(&self) -> PathBuf {
        self.db_dir().join("aira.sqlite")
    }
    pub fn artifacts(&self) -> PathBuf {
        self.root.join("artifacts")
    }
    pub fn csu_dir(&self) -> PathBuf {
        self.root.join("csu")
    }
    pub fn csu_registry(&self) -> PathBuf {
        self.csu_dir().join("registry.json")
    }
    pub fn events_dir(&self) -> PathBuf {
        self.root.join("events")
    }
    pub fn event_log(&self) -> PathBuf {
        self.events_dir().join("event-log.json")
    }

    /// Durable hash-chain event log path (QUEUE #156 / #157).
    pub fn file_chain_event_log(&self) -> PathBuf {
        self.events_dir().join("file-chain-log.json")
    }

    pub fn problems_dir(&self) -> PathBuf {
        self.root.join("problems")
    }
    pub fn problems_index(&self) -> PathBuf {
        self.problems_dir().join("index.json")
    }

    /// Durable ready-solution lookup keyed by problem-text content hash (QUEUE #189).
    pub fn reuse_index(&self) -> PathBuf {
        self.problems_dir().join("reuse-index.json")
    }
    pub fn conformance_reports(&self) -> PathBuf {
        self.root.join("conformance").join("reports")
    }
}

/// Persisted problem record for CLI status/result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemRecord {
    pub problem_id: String,
    pub text: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_artifact_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field_artifact_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProblemsIndex {
    problems: BTreeMap<String, ProblemRecord>,
}

/// Persistent map: problem-text `sha256:` hash → reusable artifact id.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReuseIndex {
    by_content_hash: BTreeMap<String, String>,
}

/// JSON event log file shape (`events/event-log.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct EventLogFile {
    pub events: Vec<EventDescriptor>,
}

/// Outcome of reading `events/event-log.json` with corruption recovery (#142 / #155).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLogReadOutcome {
    pub log: EventLogFile,
    pub recovered_from_corruption: bool,
}

/// Backup suffix when `event-log.json` is corrupt and repaired or reset.
pub const EVENT_LOG_CORRUPT_BACKUP: &str = "event-log.json.corrupt";

/// Read event log with fail-safe recovery.
///
/// 1. Missing file → empty log.
/// 2. Full JSON parse OK (no trailing junk) → as-is.
/// 3. Valid `EventLogFile` value with trailing bytes (#155) → keep value, backup + rewrite.
/// 4. Truncated/corrupt `events` array (#155) → longest prefix of valid `EventDescriptor`s.
/// 5. Otherwise (#142) → backup + empty reset.
pub fn read_event_log_resilient(path: &Path) -> Result<EventLogReadOutcome, FlowError> {
    if !path.exists() {
        return Ok(EventLogReadOutcome {
            log: EventLogFile::default(),
            recovered_from_corruption: false,
        });
    }
    let raw = fs::read_to_string(path).map_err(|e| FlowError::Other(e.to_string()))?;

    let mut de = serde_json::Deserializer::from_str(&raw);
    if let Ok(log) = EventLogFile::deserialize(&mut de) {
        if de.end().is_ok() {
            return Ok(EventLogReadOutcome {
                log,
                recovered_from_corruption: false,
            });
        }
        // Trailing bytes after a complete EventLogFile — preserve prefix (#155).
        return persist_recovered_log(path, log);
    }

    if let Some(log) = recover_event_log_events_prefix(&raw) {
        return persist_recovered_log(path, log);
    }

    // Unrecoverable: backup + empty (#142).
    backup_corrupt_log(path)?;
    let empty = EventLogFile::default();
    write_json(path, &empty)?;
    Ok(EventLogReadOutcome {
        log: empty,
        recovered_from_corruption: true,
    })
}

fn persist_recovered_log(path: &Path, log: EventLogFile) -> Result<EventLogReadOutcome, FlowError> {
    backup_corrupt_log(path)?;
    write_json(path, &log)?;
    Ok(EventLogReadOutcome {
        log,
        recovered_from_corruption: true,
    })
}

fn backup_corrupt_log(path: &Path) -> Result<(), FlowError> {
    let parent = path
        .parent()
        .ok_or_else(|| FlowError::Other("event log path has no parent directory".into()))?;
    let backup = parent.join(EVENT_LOG_CORRUPT_BACKUP);
    if backup.exists() {
        fs::remove_file(&backup).map_err(|e| FlowError::Other(e.to_string()))?;
    }
    fs::rename(path, &backup).map_err(|e| FlowError::Other(e.to_string()))?;
    Ok(())
}

/// Recover longest prefix of valid events from a damaged `{"events":[...` payload (#155).
fn recover_event_log_events_prefix(raw: &str) -> Option<EventLogFile> {
    let key = "\"events\"";
    let key_pos = raw.find(key)?;
    let after_key = &raw[key_pos + key.len()..];
    let bracket_rel = after_key.find('[')?;
    let mut cursor = &after_key[bracket_rel + 1..];
    let mut events = Vec::new();

    loop {
        cursor = cursor.trim_start();
        if cursor.is_empty() || cursor.starts_with(']') {
            break;
        }
        if cursor.starts_with(',') {
            cursor = cursor[1..].trim_start();
            continue;
        }
        match take_json_value(cursor) {
            Some((value_str, rest)) => match serde_json::from_str::<EventDescriptor>(value_str) {
                Ok(ev) => {
                    events.push(ev);
                    cursor = rest;
                }
                Err(_) => break,
            },
            None => break,
        }
    }

    // Prefix recovery only when at least one event was salvaged (empty→full reset path).
    if events.is_empty() {
        return None;
    }
    Some(EventLogFile { events })
}

/// Split one complete JSON value (object/array) from the start of `s`.
fn take_json_value(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'{') && bytes.first() != Some(&b'[') {
        return None;
    }
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            match b {
                b'\\' => escape = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' | b'[' => depth += 1,
            b'}' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[..=i], &s[i + 1..]));
                }
                if depth < 0 {
                    return None;
                }
            }
            _ => {}
        }
    }
    None
}

/// Initialize `.aira` layout (#57).
pub fn init_node(root: impl AsRef<Path>) -> Result<NodePaths, FlowError> {
    let paths = NodePaths::new(root);
    for dir in [
        paths.root.clone(),
        paths.identity_dir(),
        paths.db_dir(),
        paths.artifacts(),
        paths.csu_dir(),
        paths.events_dir(),
        paths.problems_dir(),
        paths.conformance_reports(),
        paths.root.join("models"),
    ] {
        fs::create_dir_all(&dir).map_err(|e| FlowError::Other(e.to_string()))?;
    }

    // JSON writer only when neither config.json nor config.yaml exists (YAML-only stays untouched).
    if !node_config_present(&paths.root) {
        let cfg = NodeConfig::default();
        write_json(&paths.config_json(), &cfg)?;
    }

    // Initialize SQLite object store schema.
    let _db =
        SqliteObjectStore::open(paths.sqlite()).map_err(|e| FlowError::Core(e.to_string()))?;

    // Initialize artifact CAS (creates sha256/).
    let _arts = CasArtifactStore::open(paths.artifacts())
        .map_err(|e| FlowError::Artifact(e.to_string()))?;

    if !paths.event_log().exists() {
        write_json(&paths.event_log(), &EventLogFile::default())?;
    }
    // Durable file-chain backend (#157); legacy event-log.json kept for recovery helpers.
    FileChainEventLog::open_or_create(paths.file_chain_event_log())
        .map_err(|e| FlowError::Other(e.to_string()))?;
    if !paths.problems_index().exists() {
        write_json(&paths.problems_index(), &ProblemsIndex::default())?;
    }
    if !paths.reuse_index().exists() {
        write_json(&paths.reuse_index(), &ReuseIndex::default())?;
    }
    if !paths.csu_registry().exists() {
        fs::write(paths.csu_registry(), "[]\n").map_err(|e| FlowError::Other(e.to_string()))?;
    }

    Ok(paths)
}

/// True if `config.json` and/or `config.yaml` exists under `root`.
///
/// Presence means the node is considered initialized; [`load_config`] still
/// fails closed when **both** files are present.
pub fn node_config_present(root: impl AsRef<Path>) -> bool {
    let paths = NodePaths::new(root);
    paths.config_json().exists() || paths.config_yaml().exists()
}

/// Load node config from disk (`config.json` **xor** `config.yaml`).
pub fn load_config(root: impl AsRef<Path>) -> Result<NodeConfig, FlowError> {
    let paths = NodePaths::new(root);
    let json_path = paths.config_json();
    let yaml_path = paths.config_yaml();
    match (json_path.exists(), yaml_path.exists()) {
        (true, true) => Err(FlowError::Other(
            "both config.json and config.yaml present — remove one (fail-closed)".into(),
        )),
        (true, false) => read_json(&json_path),
        (false, true) => {
            let raw =
                fs::read_to_string(&yaml_path).map_err(|e| FlowError::Other(e.to_string()))?;
            serde_norway::from_str(&raw).map_err(|e| FlowError::Other(e.to_string()))
        }
        (false, false) => Err(FlowError::Other(format!(
            "node not initialized at {} (missing config.json / config.yaml)",
            paths.root.display()
        ))),
    }
}

/// Local session: C1 reference [`OperationalPlane`] + disk persistence.
///
/// The plane remains in-process demo/conformance wiring (not production runtime).
pub struct LocalSession {
    pub paths: NodePaths,
    pub config: NodeConfig,
    plane: OperationalPlane,
}

impl LocalSession {
    /// Open an existing initialized node root.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, FlowError> {
        let paths = NodePaths::new(root);
        if !node_config_present(&paths.root) {
            return Err(FlowError::Other(format!(
                "node not initialized at {} (run `aira init`)",
                paths.root.display()
            )));
        }
        let config = load_config(&paths.root)?;
        bind_node_crypto(&paths.root)?;
        let nonce = alloc_run_nonce();
        let plane = OperationalPlane::open_with_ready_nonce(paths.artifacts(), vec![], nonce)?;
        Ok(Self {
            paths,
            config,
            plane,
        })
    }

    pub fn plane(&self) -> &OperationalPlane {
        &self.plane
    }

    pub fn plane_mut(&mut self) -> &mut OperationalPlane {
        &mut self.plane
    }

    /// Submit problem, drain pipeline, persist events + problem index.
    pub fn submit_problem(&mut self, text: &str) -> Result<SubmitOutcome, FlowError> {
        bind_node_crypto(&self.paths.root)?;
        // Allocate a fresh nonce and rebuild plane so ids never collide with prior runs.
        // Seed Reduction from the durable reuse index for this problem text (#189).
        let nonce = alloc_run_nonce();
        let ready = load_ready_solutions_for_text(&self.paths, text)?;
        self.plane = OperationalPlane::open_with_ready_nonce(self.paths.artifacts(), ready, nonce)?;
        let outcome = self.plane.submit_problem(text)?;
        self.persist_after_submit(text, &outcome)?;
        Ok(outcome)
    }

    fn persist_after_submit(
        &mut self,
        text: &str,
        outcome: &SubmitOutcome,
    ) -> Result<(), FlowError> {
        // Artifact index already flushed by CasArtifactStore::publish.
        let read = read_event_log_resilient(&self.paths.event_log())?;
        let mut log = read.log;
        for ev in self.plane.events() {
            if !log
                .events
                .iter()
                .any(|e| e.event_id.as_str() == ev.event_id.as_str())
            {
                log.events.push(ev.clone());
            }
        }
        write_json(&self.paths.event_log(), &log)?;

        // Durable hash-chain log (#157): append only new event ids.
        let mut durable = FileChainEventLog::open_or_create(self.paths.file_chain_event_log())
            .map_err(|e| FlowError::Other(e.to_string()))?;
        let known: std::collections::HashSet<String> = durable
            .chain()
            .records()
            .iter()
            .map(|r| r.event.event_id.as_str().to_string())
            .collect();
        for ev in self.plane.events() {
            if !known.contains(ev.event_id.as_str()) {
                durable
                    .append(ev.clone())
                    .map_err(|e| FlowError::Other(e.to_string()))?;
            }
        }

        // Fail-closed: corrupt index is not replaced with empty (#191).
        let mut idx = if self.paths.problems_index().exists() {
            read_json::<ProblemsIndex>(&self.paths.problems_index())
                .map_err(|e| FlowError::Other(format!("problems index: {e}")))?
        } else {
            ProblemsIndex::default()
        };
        let record = match outcome {
            SubmitOutcome::Completed {
                problem_id,
                verified_artifact_id,
                result,
            } => ProblemRecord {
                problem_id: problem_id.as_str().to_string(),
                text: text.to_string(),
                status: "completed".into(),
                verified_artifact_id: Some(verified_artifact_id.as_str().to_string()),
                field_artifact_id: None,
                result: Some(result.clone()),
            },
            SubmitOutcome::NeedsHumanCollapse { field_artifact_id } => {
                let problem_id = self
                    .plane
                    .problem_ref()
                    .map(|r| r.as_str().to_string())
                    .unwrap_or_else(|| "aira:problem:unknown".into());
                ProblemRecord {
                    problem_id: problem_id.clone(),
                    text: text.to_string(),
                    status: "needs_human_collapse".into(),
                    verified_artifact_id: None,
                    field_artifact_id: Some(field_artifact_id.as_str().to_string()),
                    result: None,
                }
            }
        };
        idx.problems.insert(record.problem_id.clone(), record);
        write_json(&self.paths.problems_index(), &idx)?;
        if let SubmitOutcome::Completed {
            verified_artifact_id,
            ..
        } = outcome
        {
            record_reuse_index(&self.paths, text, verified_artifact_id)?;
        }
        Ok(())
    }

    pub fn problem_status(&self, problem_ref: &str) -> Result<ProblemRecord, FlowError> {
        let idx = read_json::<ProblemsIndex>(&self.paths.problems_index())?;
        idx.problems
            .get(problem_ref)
            .cloned()
            .ok_or_else(|| FlowError::Other(format!("problem not found: {problem_ref}")))
    }

    pub fn get_artifact(&self, artifact_ref: &str) -> Result<(Value, Vec<u8>), FlowError> {
        let id = AiraRef::parse(artifact_ref).map_err(|e| FlowError::Other(e.to_string()))?;
        let store = CasArtifactStore::open(self.paths.artifacts())
            .map_err(|e| FlowError::Artifact(e.to_string()))?;
        let (desc, bytes) = store
            .resolve(&id)
            .map_err(|e| FlowError::Artifact(e.to_string()))?;
        let desc_v = serde_json::to_value(&desc).map_err(|e| FlowError::Other(e.to_string()))?;
        Ok((desc_v, bytes))
    }

    pub fn get_result(&self, result_ref: &str) -> Result<Value, FlowError> {
        // Accept verified artifact id or problem id.
        if let Ok(rec) = self.problem_status(result_ref) {
            if let Some(v) = rec.result {
                return Ok(v);
            }
            if let Some(aid) = rec.verified_artifact_id {
                let (_, bytes) = self.get_artifact(&aid)?;
                return serde_json::from_slice(&bytes).map_err(|e| FlowError::Other(e.to_string()));
            }
            return Err(FlowError::Other(format!(
                "no result for problem {result_ref} (status={})",
                rec.status
            )));
        }
        let (_, bytes) = self.get_artifact(result_ref)?;
        serde_json::from_slice(&bytes).map_err(|e| FlowError::Other(e.to_string()))
    }

    pub fn event_tail(&self, limit: usize) -> Result<Vec<EventDescriptor>, FlowError> {
        let chain_path = self.paths.file_chain_event_log();
        if chain_path.exists() {
            let durable = FileChainEventLog::open(&chain_path)
                .map_err(|e| FlowError::Other(e.to_string()))?;
            let events: Vec<EventDescriptor> = durable
                .chain()
                .records()
                .iter()
                .map(|r| r.event.clone())
                .collect();
            let n = events.len();
            let start = n.saturating_sub(limit);
            return Ok(events[start..].to_vec());
        }
        let read = read_event_log_resilient(&self.paths.event_log())?;
        let log = read.log;
        let n = log.events.len();
        let start = n.saturating_sub(limit);
        Ok(log.events[start..].to_vec())
    }
}

/// Open the node-layout SQLite object store (`db/aira.sqlite`) without touching the
/// in-memory [`OperationalPlane`] store (QUEUE #158).
///
/// Core keeps `SqliteObjectStore` / `MemoryObjectStore` in `aira-core`; this helper
/// only resolves the path from [`NodePaths`]. It does **not** import `aira-node`.
pub fn open_node_sqlite_object_store(paths: &NodePaths) -> Result<SqliteObjectStore, FlowError> {
    SqliteObjectStore::open(paths.sqlite()).map_err(|e| FlowError::Core(e.to_string()))
}

/// Allocate a process-safe run nonce (UUIDv7). No shared `run-counter` file.
pub(crate) fn alloc_run_nonce() -> String {
    uuid::Uuid::now_v7().simple().to_string()
}

/// Load node identity, trust, and CSU tenants. Errors are not swallowed (#190).
fn bind_node_crypto(root: &Path) -> Result<(), FlowError> {
    aira_object::register_node_identity(root).map_err(|e| FlowError::Other(e.to_string()))?;
    aira_object::ensure_trust_defaults(root).map_err(|e| FlowError::Other(e.to_string()))?;
    aira_object::load_all_csu_tenant_signing(root).map_err(|e| FlowError::Other(e.to_string()))?;
    Ok(())
}

fn problem_text_hash(text: &str) -> String {
    ContentHash::sha256_bytes(text.as_bytes())
        .as_str()
        .to_string()
}

fn load_ready_solutions_for_text(paths: &NodePaths, text: &str) -> Result<Vec<AiraRef>, FlowError> {
    if !paths.reuse_index().exists() {
        return Ok(vec![]);
    }
    let idx = read_json::<ReuseIndex>(&paths.reuse_index())?;
    let Some(id) = idx.by_content_hash.get(&problem_text_hash(text)) else {
        return Ok(vec![]);
    };
    let parsed = AiraRef::parse(id).map_err(|e| FlowError::Other(e.to_string()))?;
    let store = CasArtifactStore::open(paths.artifacts())
        .map_err(|e| FlowError::Artifact(e.to_string()))?;
    match store.resolve(&parsed) {
        Ok(_) => Ok(vec![parsed]),
        Err(_) => Ok(vec![]),
    }
}

fn record_reuse_index(
    paths: &NodePaths,
    text: &str,
    verified_artifact_id: &AiraRef,
) -> Result<(), FlowError> {
    let mut idx = if paths.reuse_index().exists() {
        read_json::<ReuseIndex>(&paths.reuse_index())?
    } else {
        ReuseIndex::default()
    };
    idx.by_content_hash
        .entry(problem_text_hash(text))
        .or_insert_with(|| verified_artifact_id.as_str().to_string());
    write_json(&paths.reuse_index(), &idx)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), FlowError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| FlowError::Other(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(value).map_err(|e| FlowError::Other(e.to_string()))?;
    let name = path
        .file_name()
        .ok_or_else(|| FlowError::Other("json path has no file name".into()))?;
    let tmp = path.with_file_name(format!("{}.tmp", name.to_string_lossy()));
    fs::write(&tmp, json).map_err(|e| FlowError::Other(e.to_string()))?;
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        FlowError::Other(e.to_string())
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, FlowError> {
    let raw = fs::read_to_string(path).map_err(|e| FlowError::Other(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| FlowError::Other(e.to_string()))
}
