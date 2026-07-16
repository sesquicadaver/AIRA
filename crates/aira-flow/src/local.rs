//! Local node layout and session persistence (Epic 8 / #57–#62).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use aira_artifact::{ArtifactStore, CasArtifactStore};
use aira_core::SqliteObjectStore;
use aira_event::EventDescriptor;
use aira_object::AiraRef;
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

    pub fn config(&self) -> PathBuf {
        self.root.join("config.json")
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
    pub fn problems_dir(&self) -> PathBuf {
        self.root.join("problems")
    }
    pub fn problems_index(&self) -> PathBuf {
        self.problems_dir().join("index.json")
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct EventLogFile {
    events: Vec<EventDescriptor>,
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
    ] {
        fs::create_dir_all(&dir).map_err(|e| FlowError::Other(e.to_string()))?;
    }

    if !paths.config().exists() {
        let cfg = NodeConfig::default();
        write_json(&paths.config(), &cfg)?;
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
    if !paths.problems_index().exists() {
        write_json(&paths.problems_index(), &ProblemsIndex::default())?;
    }
    if !paths.csu_registry().exists() {
        fs::write(paths.csu_registry(), "[]\n").map_err(|e| FlowError::Other(e.to_string()))?;
    }

    Ok(paths)
}

/// Load node config from disk.
pub fn load_config(root: impl AsRef<Path>) -> Result<NodeConfig, FlowError> {
    let paths = NodePaths::new(root);
    read_json(&paths.config())
}

/// Local session: operational plane + disk persistence.
pub struct LocalSession {
    pub paths: NodePaths,
    pub config: NodeConfig,
    plane: OperationalPlane,
}

impl LocalSession {
    /// Open an existing initialized node root.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, FlowError> {
        let paths = NodePaths::new(root);
        if !paths.config().exists() {
            return Err(FlowError::Other(format!(
                "node not initialized at {} (run `aira init`)",
                paths.root.display()
            )));
        }
        let config = load_config(&paths.root)?;
        let nonce = peek_run_nonce(&paths)?;
        let plane = OperationalPlane::open_with_ready_nonce(paths.artifacts(), vec![], nonce)?;
        // Register node identity keys (if present) into the process keyring for verify/sign.
        let _ = aira_object::register_node_identity(&paths.root);
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
        // Allocate a fresh nonce and rebuild plane so ids never collide with prior runs.
        let nonce = alloc_run_nonce(&self.paths)?;
        self.plane =
            OperationalPlane::open_with_ready_nonce(self.paths.artifacts(), vec![], nonce)?;
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
        let mut log = read_json::<EventLogFile>(&self.paths.event_log()).unwrap_or_default();
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

        let mut idx = read_json::<ProblemsIndex>(&self.paths.problems_index()).unwrap_or_default();
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
        let log = read_json::<EventLogFile>(&self.paths.event_log()).unwrap_or_default();
        let n = log.events.len();
        let start = n.saturating_sub(limit);
        Ok(log.events[start..].to_vec())
    }
}

fn run_counter_path(paths: &NodePaths) -> PathBuf {
    paths.root.join("run-counter")
}

fn peek_run_nonce(paths: &NodePaths) -> Result<u64, FlowError> {
    let path = run_counter_path(paths);
    if !path.exists() {
        return Ok(1);
    }
    let raw = fs::read_to_string(&path).map_err(|e| FlowError::Other(e.to_string()))?;
    Ok(raw.trim().parse::<u64>().unwrap_or(0).saturating_add(1))
}

fn alloc_run_nonce(paths: &NodePaths) -> Result<u64, FlowError> {
    let path = run_counter_path(paths);
    let next = peek_run_nonce(paths)?;
    fs::write(&path, format!("{next}\n")).map_err(|e| FlowError::Other(e.to_string()))?;
    Ok(next)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), FlowError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| FlowError::Other(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(value).map_err(|e| FlowError::Other(e.to_string()))?;
    fs::write(path, json).map_err(|e| FlowError::Other(e.to_string()))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, FlowError> {
    let raw = fs::read_to_string(path).map_err(|e| FlowError::Other(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| FlowError::Other(e.to_string()))
}
